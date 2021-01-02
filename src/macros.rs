#[macro_export]
macro_rules! hash_map {
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
    };
}

#[macro_export]
macro_rules! name_map {
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = crate::name_map::NameMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
    };
}
