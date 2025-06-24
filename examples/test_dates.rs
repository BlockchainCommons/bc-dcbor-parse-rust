use dcbor_parse::parse_dcbor_item;

fn main() {
    // Test parsing a simple date
    match parse_dcbor_item("2023-02-08") {
        Ok(cbor) => {
            println!("Successfully parsed date: {}", cbor.diagnostic());
            println!("CBOR: {:?}", cbor);
        }
        Err(e) => {
            println!("Error parsing date: {:?}", e);
        }
    }

    // Test parsing a date-time
    match parse_dcbor_item("2023-02-08T15:30:45Z") {
        Ok(cbor) => {
            println!("Successfully parsed datetime: {}", cbor.diagnostic());
            println!("CBOR: {:?}", cbor);
        }
        Err(e) => {
            println!("Error parsing datetime: {:?}", e);
        }
    }

    // Test parsing an array with dates
    match parse_dcbor_item("[1965-05-15, 2000-07-25, 2004-10-30]") {
        Ok(cbor) => {
            println!("Successfully parsed date array: {}", cbor.diagnostic());
            println!("CBOR: {:?}", cbor);
        }
        Err(e) => {
            println!("Error parsing date array: {:?}", e);
        }
    }
}
