fn main() {
    //hysterical::utils::testing::fetch_query_based("Win32_PhysicalMemory");
    
    let thing = hysterical::MemInfo::fetch();
    println!("{:#?}", thing)
}
