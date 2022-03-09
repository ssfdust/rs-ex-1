use std::io;

pub fn get_user_confirm() -> bool {
    let input: String = get_input("continue to next with yes: ");
    if input.contains("n") || input.contains("N") {
        return true;
    } else {
        return false;
    }
}

pub fn get_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_goes_into_input_above) => {}
        Err(_no_updates_is_fine) => {}
    }
    input.trim().to_string()
}
