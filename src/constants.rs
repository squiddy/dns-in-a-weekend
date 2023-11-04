#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]

macro_rules! make_enum {
    ($name:ident {$($vname:ident = $vvalue:expr)*}) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum $name {
            $($vname = $vvalue,)*
        }

        impl TryFrom<u16> for $name {
            type Error = anyhow::Error;

            fn try_from(v: u16) -> Result<Self, Self::Error> {
                match v {
                    $(x if x == Self::$vname as u16 => Ok(Self::$vname),)*
                    _ => Err(anyhow::anyhow!("Unknown {}: {}", stringify!($name), v)),
                }

            }
        }
    }
}

// <https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.2>
make_enum!(
    Type {
        A = 1
        NS = 2
        CNAME = 5
        SOA = 6
        HINFO = 13
        MX = 15
        TXT = 16
        AAAA = 28
        ALL_RECORDS = 255
    }
);

// <https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.4>
make_enum!(
    Class {
        IN = 1
    }
);

/// <https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1>
pub enum Flags {
    RECURSION_DESIRED = 1 << 8,
}

/// <https://www.iana.org/domains/root/servers>
pub static ROOT_SERVERS_V4: &[&str] = &[
    "198.41.0.4",
    "199.9.14.201",
    "192.33.4.12",
    "199.7.91.13",
    "192.203.230.10",
    "192.5.5.241",
    "192.112.36.4",
    "198.97.190.53",
    "192.36.148.17",
    "192.58.128.30",
    "193.0.14.129",
    "199.7.83.42",
    "202.12.27.33",
];
