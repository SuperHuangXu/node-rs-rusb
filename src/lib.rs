#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;
use core::result;
use napi::{CallContext, Env, JsNumber, JsObject, Result, Task};
use rusb::{self, Error};
use std::convert::TryInto;
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

fn get_device_list() -> result::Result<Vec<Device>, Error> {
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

struct AsyncTask();

impl Task for AsyncTask {
    type Output = Vec<Device>;
    type JsValue = JsObject;

    fn compute(&mut self) -> Result<Self::Output> {
        let list = get_device_list().unwrap();
        Ok(list)
    }

    fn resolve(self, env: Env, output: Self::Output) -> Result<Self::JsValue> {
        let mut arr = env.create_array()?;
        for (index, device) in output.into_iter().enumerate() {
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
}

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
    exports.create_named_method("getDeviceList", get_device_list_fn)?;
    Ok(())
}

#[js_function(1)]
fn sync_fn(ctx: CallContext) -> Result<JsNumber> {
    let argument: u32 = ctx.get::<JsNumber>(0)?.try_into()?;

    ctx.env.create_uint32(argument + 100)
}

#[js_function(1)]
fn get_device_list_fn(ctx: CallContext) -> Result<JsObject> {
    let task = AsyncTask();
    let async_task = ctx.env.spawn(task)?;
    Ok(async_task.promise_object())
}
