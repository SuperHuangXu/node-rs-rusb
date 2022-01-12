#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;
use core::result;
use napi::{CallContext, Env, JsObject, Result, Task};
use rusb;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug)]
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
        let device_desc = device.device_descriptor()?;
        let device_handle = device.open()?;
        let timeout = Duration::from_secs(1);
        let languages = device_handle.read_languages(timeout)?;
        let language = languages[0];

        list.push(Device {
            vendor_id: device_desc.vendor_id(),
            product_id: device_desc.product_id(),
            manufacturer: device_handle.read_manufacturer_string(
                language,
                &device_desc,
                timeout,
            )?,
            serial_number: device_handle
                .read_serial_number_string(language, &device_desc, timeout)
                .unwrap_or_default(),
            device_address: device.address(),
            device_name: device_handle
                .read_product_string(language, &device_desc, timeout)
                .unwrap_or_default(),
        });
    }
    Ok(list)
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
        if let Some(params) = params {
            if let Some(pid) = params.pid {
                list = list
                    .into_iter()
                    .filter(|device| device.product_id == pid as u16)
                    .collect::<Vec<_>>();
            }
            if let Some(vid) = params.vid {
                list = list
                    .into_iter()
                    .filter(|device| device.vendor_id == vid as u16)
                    .collect::<Vec<_>>();
            }
        }
        Ok(list)
    }

    fn resolve(self, env: Env, output: Self::Output) -> Result<Self::JsValue> {
        convert_to_js(&env, &output)
    }
}

fn convert_to_js(env: &Env, devices: &Vec<Device>) -> Result<JsObject> {
    let mut arr = env.create_array()?;
    for (index, device) in devices.into_iter().enumerate() {
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
        arr.set_element(index as u32, o)?;
    }
    Ok(arr)
}

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
    exports.create_named_method("find", find_fn)?;
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
