use abi_stable::{rvec, std_types::{RString, RVec}};


// 3 common categories

pub fn array_sources()->RVec<RString>{
    rvec!["Array/Signal sources".into()]
}

pub fn time_sources()->RVec<RString>{
    rvec!["Time sources".into()]
}

pub fn data_sources()->RVec<RString>{
    rvec!["Data sources".into()]
}

pub fn data_savers()->RVec<RString>{
    rvec!["Data savers".into()]
}

pub fn array_savers()->RVec<RString>{
    rvec!["Array savers".into()]
}
