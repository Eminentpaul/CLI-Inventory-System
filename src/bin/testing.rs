
fn generate_account_number(phone_no: &str) -> String {
    let phone_no = phone_no.trim();
    let number = phone_no.len() - 10;
    let new_number = &phone_no[number..];
    String::from(new_number)
}


fn main(){
    println!("Acct. Nu: {}", generate_account_number("08143122946\r\n"))
}