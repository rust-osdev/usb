#![no_std]
#![feature(slice_as_chunks)]

pub mod descriptor;

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
    pub const DIR_OUT: u8 = 0 << 7;
    pub const DIR_IN: u8 = 1 << 7;

    pub const TYPE_STANDARD: u8 = 0 << 5;
    pub const TYPE_CLASS: u8 = 1 << 5;
    pub const TYPE_VENDOR: u8 = 2 << 5;

    pub const RECIPIENT_DEVICE: u8 = 0;
    pub const RECIPIENT_INTERFACE: u8 = 1;
    pub const RECIPIENT_ENDPOINT: u8 = 2;
    pub const RECIPIENT_OTHER: u8 = 3;

    pub const GET_STATUS: u8 = 0;
    pub const CLEAR_FEATURE: u8 = 1;
    pub const SET_FEATURE: u8 = 3;
    pub const SET_ADDRESS: u8 = 5;
    pub const GET_DESCRIPTOR: u8 = 6;
    pub const SET_DESCRIPTOR: u8 = 7;
    pub const GET_CONFIGURATION: u8 = 8;
    pub const SET_CONFIGURATION: u8 = 9;
    pub const GET_INTERFACE: u8 = 10;
    pub const SET_INTERFACE: u8 = 11;
    pub const SYNC_FRAME: u8 = 12;

    pub fn direction_in(&self) -> bool {
        self.request_type & Self::DIR_IN != 0
    }
}

impl From<Request> for RawRequest {
    fn from(r: Request) -> Self {
        let w_value = |ty, i| u16::from(ty) << 8 | u16::from(i);
        match r {
            Request::GetDescriptor { ty } => {
                use descriptor::GetDescriptor::*;
                RawRequest {
                    request_type: Self::DIR_IN
                        | Self::TYPE_STANDARD
                        | match ty {
                            Device | Configuration { .. } | String { .. } => Self::RECIPIENT_DEVICE,
                            Report => Self::RECIPIENT_INTERFACE,
                        },
                    request: Self::GET_DESCRIPTOR,
                    value: match ty {
                        Device => w_value(descriptor::DEVICE, 0),
                        Configuration { index } => w_value(descriptor::CONFIGURATION, index),
                        String { index } => w_value(descriptor::STRING, index),
                        Report => w_value(descriptor::REPORT, 0),
                    },
                    index: 0,
                }
            }
            Request::SetConfiguration { value } => RawRequest {
                request_type: Self::DIR_OUT | Self::TYPE_STANDARD | Self::RECIPIENT_DEVICE,
                request: Self::SET_CONFIGURATION,
                value: value.into(),
                index: 0,
            },
            _ => todo!(),
        }
    }
}
