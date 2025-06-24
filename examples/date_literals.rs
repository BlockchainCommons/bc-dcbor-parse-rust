use dcbor_parse::parse_dcbor_item;

fn main() {
    // Register dcbor tags so dates are handled properly
    dcbor::register_tags();

    println!("=== dCBOR Date Literal Parsing Demo ===\n");

    // Test the exact example from the user's request
    let input = "[1965-05-15, 2000-07-25, 2004-10-30]";
    println!("Input: {}", input);

    match parse_dcbor_item(input) {
        Ok(cbor) => {
            println!("✅ Successfully parsed!");
            println!("Diagnostic output: {}", cbor.diagnostic());
            println!(
                "Note: The '1(...)' indicates CBOR tag 1 (dates), not strings!"
            );
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
        }
    }

    println!("\n=== More Examples ===\n");

    let examples = vec![
        "2023-02-08",                                  // Simple date
        "2023-02-08T15:30:45Z",                        // Date with time
        "2023-02-08T15:30:45.123Z", // Date with milliseconds
        r#"{"start": 2023-01-01, "end": 2023-12-31}"#, // Map with dates
    ];

    for example in examples {
        println!("Input: {}", example);
        match parse_dcbor_item(example) {
            Ok(cbor) => {
                println!("✅ Output: {}", cbor.diagnostic());
            }
            Err(e) => {
                println!("❌ Error: {:?}", e);
            }
        }
        println!();
    }
}
