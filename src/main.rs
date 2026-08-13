#![allow(unused)]

use std::{format, io::{self, BufReader, BufWriter}, println};
use serde::{Serialize, Deserialize};

use argon2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use rand::rngs::OsRng;
use std::fs::File;

// User Types 
#[derive(Serialize, Deserialize, Debug, Clone)]
enum UserType {
    Admin,
    Guest
}


#[derive(Serialize, Deserialize, Debug, Clone)]
enum Category {
    Food,
    Electronics,
    Fashion,
    Grocery,
    Computing
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    name: String,
    address: String,
    email: String,
    user_type: UserType,
    // password_hash: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Account {
    user: User,
    account_number: String,
    balance: f64
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Product {
    name: String,
    price: f64,
    stock: i32,
    category: Category
}

impl User {
    fn new(name:&str, address:&str, email:&str) -> Self {
        // let hashed = hash_password(password);

        Self { 
            name: String::from(name), 
            address: String::from(address), 
            email: String::from(email), 
            user_type: UserType::Guest
            
         }
    }
}


impl Account {
    fn new(user: &User, phone_no:&str) -> Self {
        let acc_no = generate_account_number(phone_no);

        Self { user: user.clone(), account_number: acc_no, balance: 0.0 }
    }
}


fn main() {
    println!("INVENTORY SYSTEM");
    let acct_path = "account.json";

    loop {
        println!("Options: \n1. Add Product\n2. View All Products\n3. View Single Product\n4. Restock Product\n5. Sell Product\n6. Update Product\n7. Delete Product\n8. Save Inventory\n9. View Cart\n10. View Orders\n11. Create Account\n11. Exit");

        let input = user_input("Please Select an Option:");

        match input.trim() {
            "11" => {
                let full_name = user_input("Enter Full Name:");
                let email = user_input("Enter Your Email:");
                let address = user_input("Enter your Full Address:");
                // let password = user_input("Enter Account Password");
                let phone_no = user_input("Enter Phone Number");
                let path = acct_path;

                let phone_no = match check_phone_number(&phone_no) {
                    Ok(phone) => phone,
                    Err(_) => {return;}
                };

                create_account(&full_name, &email, &address, &phone_no, path);
            },

            _ => {
                println!("Invalid Input")
            }
        }
    }
}


fn create_account(name:&str, email:&str, address:&str, phone_no:&str, path:&str) {
    
    let db:Vec<Account> = match load_account(path) {
        Ok(account) => account,
        Err(_) => {return;}
    };

    
    for account in db.iter() {
        println!("Database: {:?}", db);
        if account.account_number == generate_account_number(phone_no) {
            println!("This Account Already Existed.");
            return;
        }else {
            println!("Normal");
            let user = User::new(name, address, email);
            let acct = Account::new(&user, phone_no);

            let saved: bool = match save_file(path, &db) {
                Ok(saved) => saved,
                Err(_) => {return;}
            };
             println!("Working");

            if saved{
                println!("Account Created Successfully!")
            }else {
                println!("Account not Created Successfully!")
            }
        }
    }
}



















fn hash_password(password: &str) -> String {
    println!("Hashing");
    let salt = SaltString::generate(OsRng);

    Argon2::default()
    .hash_password(password.as_bytes(), &salt)
    .unwrap()
    .to_string()
}


fn verify_password(password: &str, hash:&str) -> bool {

    let parsed_hash = match PasswordHash::new(hash) {
        Ok(hash) => hash,
        Err(_) => return false
    };


    Argon2::default()
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok()
}


fn generate_account_number(phone_no:&str) -> String {
    let number = phone_no.len()-10;
    let new_number = &phone_no[number..];
    String::from(new_number)
}


fn check_phone_number(phone_no:&str) -> Result<String, String> {
     let phone_no = phone_no.trim();

    if !phone_no.chars().all(|c| c.is_ascii_digit()) {
        return Err("Phone number should contain only digits".to_string());
    }

    if !phone_no.starts_with("0") {
        return Err("Phone Number must start with zeror".to_string());
    }

    if phone_no.len() != 11 {
        return Err("Phone number must be 11 digits".to_string());
    }



    Ok(phone_no.to_string())

}

fn user_input(option: &str) -> String {
    println!("{}", option);

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("No Input Found!");


    input
}


fn load_account<T: serde::de::DeserializeOwned> (path:&str) -> Result<Vec<T>, String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            let error = format!("Open File Error: {}", err);
            return Ok(Vec::new());
        }
    };

    let reader = BufReader::new(file);

    let account: Vec<T> = match serde_json::from_reader(reader) {
        Ok(acct) => acct,
        Err(err) => {
            println!("Cannot read Account file with the following Error: {}", err);
            return Ok(Vec::new());
        }
    };

    Ok(account)


}

fn save_file<T: Serialize>(path:&str, database: &Vec<T> ) -> Result<bool, String> {
    let file = match File::create(path) {
        Ok(file) => file,
        Err(err) => {
            let error = format!("Failed with the Error: {}", err);
            return Err(error.to_string());
        }
    };


    let writer = BufWriter::new(file);

    match serde_json::to_writer_pretty(writer, &database) {
        Ok(b) => Ok(true),
        Err(_) => Err("File not saved!".to_string())

    }
}