use std::net::IpAddr;

pub type IpFilter<'a> = Box<dyn Fn(&IpAddr, &IpAddr) -> bool + Send + Sync + 'a>;

pub struct IpFilters<'a> {
    filters: Vec<IpFilter<'a>>,
}

impl<'a> Default for IpFilters<'a> {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl<'a> IpFilters<'a> {
    pub fn new(filters: Vec<IpFilter<'a>>) -> Self {
        Self { filters }
    }

    pub fn with_non_broadcast() -> Self {
        Self::new(vec![Box::new(|src, dst| {
            macro_rules! non_broadcast {
                ($addr:expr) => {
                    match $addr {
                        IpAddr::V4(v4) => {
                            !(v4.is_broadcast() || v4.is_multicast() || v4.is_multicast())
                        }
                        IpAddr::V6(v6) => !(v6.is_multicast() || v6.is_unspecified()),
                    }
                };
            }
            non_broadcast!(src) && non_broadcast!(dst)
        })])
    }

    pub fn add(&mut self, filter: IpFilter<'a>) {
        self.filters.push(filter);
    }

    pub fn add_fn<F>(&mut self, filter: F)
    where
        F: Fn(&IpAddr, &IpAddr) -> bool + Send + Sync + 'a,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn is_allowed(&self, src: &IpAddr, dst: &IpAddr) -> bool {
        for filter in &self.filters {
            if !filter(src, dst) {
                return false;
            }
        }
        true
    }
}
