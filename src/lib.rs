#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;
use core::result;
use lazy_static::lazy_static;
use napi::{
    threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode},
    CallContext, Env, JsFunction, JsObject, JsUndefined, Result, Task,
};
use rusb::{self, UsbContext};
use serde::{Deserialize, Serialize};
use std::{
    sync::{mpsc, Mutex},
    thread,
    time::Duration,
};

lazy_static! {
    static ref IS_RUNNING: Mutex<bool> = Mutex::new(false);
}

#[derive(Debug, Clone)]
struct Device {
    vendor_id: u16,
    product_id: u16,
    device_name: String,
    manufacturer: String,
    serial_number: String,
    device_address: u8,
}

fn get_device_list() -> result::Result<Vec<Device>, rusb::Error> {
    let mut list: Vec<Device> = vec![];

    for device in rusb::devices().unwrap().iter() {
        list.push(rusb_device_convert_to_device(device)?);
    }
    Ok(list)
}

fn rusb_device_convert_to_device<T>(device: rusb::Device<T>) -> result::Result<Device, rusb::Error>
where
    T: rusb::UsbContext,
{
    let device_desc = device.device_descriptor()?;
    let device_handle = device.open()?;
    let timeout = Duration::from_secs(1);
    let languages = device_handle.read_languages(timeout)?;
    let language = languages[0];

    let d = Device {
        vendor_id: device_desc.vendor_id(),
        product_id: device_desc.product_id(),
        manufacturer: device_handle.read_manufacturer_string(language, &device_desc, timeout)?,
        serial_number: device_handle
            .read_serial_number_string(language, &device_desc, timeout)
            .unwrap_or_default(),
        device_address: device.address(),
        device_name: device_handle
            .read_product_string(language, &device_desc, timeout)
            .unwrap_or_default(),
    };
    Ok(d)
}

#[derive(Serialize, Debug, Deserialize)]
struct FindByIdAsyncTaskParams {
    pid: Option<i32>,
    vid: Option<i32>,
}

struct FindAsyncTask(Option<FindByIdAsyncTaskParams>);

impl Task for FindAsyncTask {
    type Output = Vec<Device>;
    type JsValue = JsObject;

    fn compute(&mut self) -> Result<Self::Output> {
        let mut list = get_device_list().unwrap();
        let params = &self.0;
        if let Some(FindByIdAsyncTaskParams { pid, vid }) = params {
            if let Some(pid) = *pid {
                list = list
                    .into_iter()
                    .filter(|device| device.product_id == pid as u16)
                    .collect();
            }
            if let Some(vid) = *vid {
                list = list
                    .into_iter()
                    .filter(|device| device.vendor_id == vid as u16)
                    .collect();
            }
        }
        Ok(list)
    }

    fn resolve(self, env: Env, output: Self::Output) -> Result<Self::JsValue> {
        let mut arr = env.create_array()?;
        for (index, device) in output.into_iter().enumerate() {
            arr.set_element(index as u32, convert_to_js(&env, &device)?)?;
        }
        Ok(arr)
    }
}

fn convert_to_js(env: &Env, device: &Device) -> Result<JsObject> {
    let mut o = env.create_object()?;
    o.set_property(
        env.create_string("vendorId")?,
        env.create_int32(device.vendor_id as i32)?,
    )?;
    o.set_property(
        env.create_string("productId")?,
        env.create_int32(device.product_id as i32)?,
    )?;
    o.set_property(
        env.create_string("deviceName")?,
        env.create_string(&device.device_name)?,
    )?;
    o.set_property(
        env.create_string("manufacturer")?,
        env.create_string(&device.manufacturer)?,
    )?;
    o.set_property(
        env.create_string("serialNumber")?,
        env.create_string(&device.serial_number)?,
    )?;
    o.set_property(
        env.create_string("deviceAddress")?,
        env.create_int32(device.device_address as i32)?,
    )?;
    Ok(o)
}

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
    exports.create_named_method("find", find_fn)?;
    exports.create_named_method("startMonitoring", start_monitoring_fn)?;
    exports.create_named_method("stopMonitoring", stop_monitoring_fn)?;
    Ok(())
}

#[js_function(1)]
fn find_fn(ctx: CallContext) -> Result<JsObject> {
    let arg0 = ctx.get::<JsObject>(0)?;
    let params: Option<FindByIdAsyncTaskParams> = ctx.env.from_js_value(arg0)?;
    let task = FindAsyncTask(params);
    let async_task = ctx.env.spawn(task)?;
    Ok(async_task.promise_object())
}

enum Action {
    Arrived,
    Left,
}

#[derive(Debug)]
struct DeviceLess {
    vendor_id: u16,
    product_id: u16,
}

#[derive(Debug)]
enum MessageDeviceData {
    Device(Device),
    DeviceLess(DeviceLess),
}

struct Message {
    action: Action,
    device: MessageDeviceData,
}

struct SenderData<T: rusb::UsbContext> {
    action: Action,
    device: rusb::Device<T>,
}

struct HotPlugHandler<T: rusb::UsbContext>(mpsc::Sender<SenderData<T>>);

impl<T: rusb::UsbContext> rusb::Hotplug<T> for HotPlugHandler<T> {
    fn device_arrived(&mut self, device: rusb::Device<T>) {
        let data = SenderData {
            action: Action::Arrived,
            device,
        };
        self.0.send(data).unwrap();
    }

    fn device_left(&mut self, device: rusb::Device<T>) {
        let data = SenderData {
            action: Action::Left,
            device,
        };
        self.0.send(data).unwrap();
    }
}

impl<T: rusb::UsbContext> Drop for HotPlugHandler<T> {
    fn drop(&mut self) {}
}

#[js_function(1)]
fn start_monitoring_fn(ctx: CallContext) -> Result<JsUndefined> {
    let cb = ctx.get::<JsFunction>(0)?;
    let tscb =
        ctx.env
            .create_threadsafe_function(&cb, 0, |tscx: ThreadSafeCallContext<Message>| {
                let action = match &tscx.value.action {
                    Action::Arrived => "arrived",
                    Action::Left => "left",
                };
                match tscx.value.device {
                    MessageDeviceData::Device(device) => {
                        let device = convert_to_js(&tscx.env, &device)?;
                        let mut data = tscx.env.create_object()?;

                        data.set_named_property("action", tscx.env.create_string(action)?)?;
                        data.set_named_property("device", device)?;
                        Ok(vec![data])
                    }
                    MessageDeviceData::DeviceLess(device) => {
                        let mut data = tscx.env.create_object()?;
                        data.set_named_property("action", tscx.env.create_string(action)?)?;
                        let mut device_js = tscx.env.create_object()?;
                        device_js.set_named_property(
                            "productId",
                            tscx.env.create_int32(device.product_id as i32)?,
                        )?;
                        device_js.set_named_property(
                            "vendorId",
                            tscx.env.create_int32(device.vendor_id as i32)?,
                        )?;
                        data.set_named_property("device", device_js)?;
                        Ok(vec![data])
                    }
                }
            })?;
    start_monitoring(tscb).unwrap();
    ctx.env.get_undefined()
}

fn start_monitoring(tscb: ThreadsafeFunction<Message>) -> rusb::Result<()> {
    if rusb::has_hotplug() {
        let context = rusb::Context::new()?;
        let (sender, receiver) = mpsc::channel::<SenderData<_>>();

        let mut reg = Some(
            rusb::HotplugBuilder::new()
                .enumerate(false)
                .register(&context, Box::new(HotPlugHandler(sender)))?,
        );

        *IS_RUNNING.lock().unwrap() = true;
        let context_clone = context.clone();
        thread::spawn(move || loop {
            context_clone.handle_events(None).unwrap();
            if let Ok(recv) = receiver.recv() {
                match recv.action {
                    Action::Arrived => {
                        let device = rusb_device_convert_to_device(recv.device).unwrap();

                        let msg = Message {
                            action: recv.action,
                            device: MessageDeviceData::Device(device),
                        };
                        tscb.call(Ok(msg), ThreadsafeFunctionCallMode::NonBlocking);
                    }
                    Action::Left => {
                        let descriptor = recv.device.device_descriptor().unwrap();
                        let msg = Message {
                            action: recv.action,
                            device: MessageDeviceData::DeviceLess(DeviceLess {
                                vendor_id: descriptor.vendor_id(),
                                product_id: descriptor.product_id(),
                            }),
                        };
                        tscb.call(Ok(msg), ThreadsafeFunctionCallMode::NonBlocking);
                    }
                }
            } else {
                break;
            }
        });

        thread::spawn(move || loop {
            if !*IS_RUNNING.lock().unwrap() {
                let take = reg.take();
                if let Some(t) = take {
                    context.unregister_callback(t);
                }
            }
        });
        Ok(())
    } else {
        eprint!("libusb hotplug api unsupported");
        Ok(())
    }
}

#[js_function(0)]
fn stop_monitoring_fn(ctx: CallContext) -> Result<JsUndefined> {
    stop_monitoring().unwrap();
    Ok(ctx.env.get_undefined()?)
}

fn stop_monitoring() -> rusb::Result<()> {
    *IS_RUNNING.lock().unwrap() = false;
    Ok(())
}
