#![no_std] // nie ładuj biblioteki std

macro_rules! mem_array {
    ($var_name:ident, $start_addr:expr, $size:expr) => {
        let $var_name: &mut [u8] = unsafe {
            slice::from_raw_parts_mut($start_addr as *mut u8, $size)
        };    
    }
}

macro_rules! volatile_mem_array {
    ($var_name:ident, $start_addr:expr, $size:expr) => {

        let $var_name: &mut [Volatile<u8>] = core::slice::from_raw_parts_mut($start_addr as *mut Volatile<u8>, $size);


        // let $var_name: &mut [Volatile<u8>] = unsafe {
        //     slice::from_raw_parts_mut($start_addr as *mut Volatile<u8>, $size)
        // };    
    }
}
