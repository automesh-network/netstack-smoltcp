use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub type Ipv4Filter<'a> = Box<dyn Fn(&Ipv4Addr) -> bool + Send + Sync + 'a>;
pub type Ipv6Filter<'a> = Box<dyn Fn(&Ipv6Addr) -> bool + Send + Sync + 'a>;

pub struct Filters<'a> {
    ipv4_filters: Vec<Ipv4Filter<'a>>,
    ipv6_filters: Vec<Ipv6Filter<'a>>,
}

impl<'a> Default for Filters<'a> {
    fn default() -> Self {
        Self::new(
            vec![
                Box::new(|v4| !v4.is_broadcast()),
                Box::new(|v4| !v4.is_multicast()),
                Box::new(|v4| !v4.is_unspecified()),
            ],
            vec![
                Box::new(|v6| !v6.is_multicast()),
                Box::new(|v6| !v6.is_unspecified()),
            ],
        )
    }
}

impl<'a> Filters<'a> {
    pub fn new(ipv4_filters: Vec<Ipv4Filter<'a>>, ipv6_filters: Vec<Ipv6Filter<'a>>) -> Self {
        Self {
            ipv4_filters,
            ipv6_filters,
        }
    }

    pub fn is_allowed(&self, addr: &IpAddr) -> bool {
        match addr {
            IpAddr::V4(v4) => {
                for filter in &self.ipv4_filters {
                    if !filter(v4) {
                        return false;
                    }
                }
            }
            IpAddr::V6(v6) => {
                for filter in &self.ipv6_filters {
                    if !filter(v6) {
                        return false;
                    }
                }
            }
        }
        true
    }

    #[allow(unused)]
    pub fn add_v4(&mut self, filter: Ipv4Filter<'a>) {
        self.ipv4_filters.push(filter);
    }

    #[allow(unused)]
    pub fn add_v6(&mut self, filter: Ipv6Filter<'a>) {
        self.ipv6_filters.push(filter);
    }
}
