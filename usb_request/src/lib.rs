#![no_std]
#![feature(slice_as_chunks)]

pub mod descriptor;

#[allow(dead_code)]
const GET_STATUS: u8 = 0;
#[allow(dead_code)]
const CLEAR_FEATURE: u8 = 1;
#[allow(dead_code)]
const SET_FEATURE: u8 = 3;
#[allow(dead_code)]
const SET_ADDRESS: u8 = 5;
const GET_DESCRIPTOR: u8 = 6;
#[allow(dead_code)]
const SET_DESCRIPTOR: u8 = 7;
#[allow(dead_code)]
const GET_CONFIGURATION: u8 = 8;
const SET_CONFIGURATION: u8 = 9;
#[allow(dead_code)]
const GET_INTERFACE: u8 = 10;
#[allow(dead_code)]
const SET_INTERFACE: u8 = 11;
#[allow(dead_code)]
const SYNC_FRAME: u8 = 12;

#[derive(Debug)]
pub enum Request {
    GetDescriptor { ty: descriptor::GetDescriptor },
    SetConfiguration { value: u8 },
    GetReport { id: u8 },
    SetReport,
    GetIdle,
    SetIdle,
    SetProtocol,
    GetProtocol,
}

pub struct RawRequest {
    pub request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
}

impl RawRequest {
    pub fn direction_in(&self) -> bool {
        self.request_type & request_type::DIR_IN != 0
    }
}

mod request_type {
    pub const DIR_OUT: u8 = 0 << 7;
    pub const DIR_IN: u8 = 1 << 7;

    pub const TYPE_STANDARD: u8 = 0 << 5;
    #[allow(dead_code)]
    pub const TYPE_CLASS: u8 = 1 << 5;
    #[allow(dead_code)]
    pub const TYPE_VENDOR: u8 = 2 << 5;

    pub const RECIPIENT_DEVICE: u8 = 0;
    pub const RECIPIENT_INTERFACE: u8 = 1;
    #[allow(dead_code)]
    pub const RECIPIENT_ENDPOINT: u8 = 2;
    #[allow(dead_code)]
    pub const RECIPIENT_OTHER: u8 = 3;
}

impl Request {
    pub fn into_raw(self) -> RawRequest {
        use request_type::*;
        let w_value = |ty, i| u16::from(ty) << 8 | u16::from(i);
        match self {
            Self::GetDescriptor { ty } => {
                use descriptor::GetDescriptor::*;
                RawRequest {
                    request_type: DIR_IN
                        | TYPE_STANDARD
                        | match ty {
                            Device | Configuration { .. } | String { .. } => RECIPIENT_DEVICE,
                            Report => RECIPIENT_INTERFACE,
                        },
                    request: GET_DESCRIPTOR,
                    value: match ty {
                        Device => w_value(descriptor::DEVICE, 0),
                        Configuration { index } => w_value(descriptor::CONFIGURATION, index),
                        String { index } => w_value(descriptor::STRING, index),
                        Report => w_value(descriptor::REPORT, 0),
                    },
                    index: 0,
                }
            }
            Self::SetConfiguration { value } => RawRequest {
                request_type: DIR_OUT | TYPE_STANDARD | RECIPIENT_DEVICE,
                request: SET_CONFIGURATION,
                value: value.into(),
                index: 0,
            },
            _ => todo!(),
        }
    }
}
