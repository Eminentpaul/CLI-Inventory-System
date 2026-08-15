#![allow(unused)]

use std::{format, io::{self, BufReader, BufWriter}, println, ptr::null};
use serde::{Serialize, Deserialize};

use argon2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use rand::{random, rngs::OsRng, seq::index};
use std::fs::File;
use validator::{ValidateEmail, Validate};

// User Types 
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
enum UserType {
    Admin,
    Guest
}


// Activeness 
#[derive(Serialize, PartialEq, Deserialize, Debug, Clone)]
enum ActiveStatus {
    LoggedIn,
    LoggedOut
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
    password_hash: String,
    status: ActiveStatus
}

impl User {
    fn new(name:&str, address:&str, email:&str, password: &str) -> Self {
        let hashed = hash_password(password);

        Self { 
            name: String::from(name), 
            address: String::from(address), 
            email: String::from(email), 
            user_type: UserType::Guest, 
            password_hash: hashed,
            status: ActiveStatus::LoggedOut
         }
    }
}




#[derive(Serialize, Deserialize, Debug, Clone)]
struct Account {
    user: User,
    account_number: String,
    balance: f64
}

impl Account {
    fn new(user: &User, phone_no:&str) -> Self {
        let acc_no = generate_account_number(phone_no);

        Self { user: user.clone(), account_number: acc_no, balance: 0.0 }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Product {
    name: String,
    price: f64,
    stock: i32,
    category: Category
}




impl Product {
    fn new(name:&str, price: f64, stock:i32, category: Category) -> Self {
        Self { 
            name: String::from(name), 
            price, 
            stock, 
            category
         }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Cart {
    user: Account,
    product: Product,
    quantity: i32,
    active: bool
}

impl Cart {
    fn new(user: &Account, product: &Product, quantity:i32) -> Self {
        Self { 
            user: user.clone(), 
            product: product.clone(), 
            quantity: quantity, 
            active: true
         }
    }
}


fn main() {
    println!("INVENTORY SYSTEM");
    let acct_path = "account.json";
    let product_path = "product.json";
    let cart_path = "cart.json";

    let mut logged_in_account: Option<Account> = None;
    loop {

        println!("Options: \n1. Add Product\n2. View All Products\n3. View Single Product\n4. View All Cart\n5. View My Cart\n6. View All Orders\n7. View My Orders\n8. Create Account\n9. Change User Type\n10. Login\n11. Exit");

        let input = user_input("Please Select an Option:");

        match input.trim() {
            "1" => {
                println!("Add Product:");
                let active_account = match verify_login(logged_in_account.clone()) {
                    Ok(account) => account,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                let mut product_db: Vec<Product> = match load_database(product_path) {
                    Ok(product) => product,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                let product_name = user_input("Enter Product Name:");
                let price = user_input("Enter Price:");

                let price = match check_price(&price) {
                    Ok(num) => num,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                let stock = user_input("Enter the number of Stocks:");
                let stock = match check_stock(&stock) {
                    Ok(num) => num,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                println!("Select category:\n1. Food\n2. Electronics\n3. Fashion\n4. Grocery\n5. Computing");

                let cate = user_input("Select Category");
                let category = match get_category(&cate) {
                    Some(cat) => cat,
                    None => {
                        println!("Invalid Category");
                        return;
                    }
                };

                if active_account.user.user_type == UserType::Admin {
                    add_product(price, stock, &product_name, category, product_path);

                }else {
                    println!("Only Admin can perform this action")
                }

                


            },

            "2" => {
                println!("\nAll Available Product:");
                view_all_product(product_path);
            },

            "3" => {
                println!("\nAll Available Product:");
                view_all_product(product_path);

                let product_input = user_input("Select a Product:");
                let serial:usize = match product_input.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("Invalid Input");
                        return;
                    }
                };

                let mut product = match view_product(&product_path, serial) {
                    Ok(prod) => prod,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                product_output(&product, serial-1);


                println!("Product Options:\n1. Restock Product\n2. Update Product\n3. Add to Cart\n4. Delete Product");
                let input = user_input("Select an Option:");

                single_product_commands(input, serial, product, cart_path, product_path, &logged_in_account);
            },

            "4" => {
                let active_account = match verify_login(logged_in_account.clone()) {
                    Ok(account) => account,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                if active_account.user.user_type == UserType::Admin {
                    view_all_cart(cart_path);
                }else {
                    println!("Only Admin can perform this action")
                }
            },

            "8" => {
                let full_name = user_input("Enter Full Name:");
                let email = user_input("Enter Your Email:");

                // Email validation 
                if !validate_email(&email) {
                    println!("Invalid Email");
                    return;
                }

                let address = user_input("Enter your Full Address:");


                let password = user_input("Enter Account Password");

                // Password validation 
                 match validate_password(&password) {
                    Ok(pass) => pass,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                let phone_no = user_input("Enter Phone Number");
                let path = acct_path;

                let phone_no = match check_phone_number(&phone_no) {
                    Ok(phone) => phone,
                    Err(_) => {return;}
                };
                
                create_account(&full_name, &email, &address, &password, &phone_no, path);
                
            },

            "9" => {
                match change_user_type(&logged_in_account, acct_path) {
                    Ok(done) => {
                        println!("{}", done)
                    },
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };
            },

            "10" => {
                println!("Account Login");
                let phone_no = user_input("Enter Your Phone Number:");
                let phone_no = match check_phone_number(&phone_no) {
                    Ok(phone) => phone,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };
                let password = user_input("Enter Password");

                 logged_in_account = match login(&phone_no, &password, acct_path) {
                    Ok(acct) => Some(acct),
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                // println!("Account acive: {:#?}", logged_in_account);
            },

            "11" => {
                println!("Exiting.....");
                break;
            }

            _ => {
                println!("Invalid Input")
            }
        }
    }
}


fn create_account(name:&str, email:&str, address:&str, password: &str, phone_no:&str, path:&str) {
    
    let mut db:Vec<Account> = match load_database(path) {
        Ok(account) => account,
        Err(_) => {return;}
    };

   

    for account in db.iter() {
        if account.account_number == generate_account_number(phone_no) {
            println!("This Account Already Existed.");
            return;
        }
    }

    let user = User::new(name, address, email, password);
    let acct = Account::new(&user, phone_no);

    db.push(acct);

    let saved: bool = match save_file(path, &db) {
        Ok(saved) => saved,
        Err(_) => {return;}
    };


    if saved{
        println!("Account Created Successfully!")
    }else {
        println!("Account not Created Successfully!")
    }
}


fn login(phone_no: &str, password: &str, path:&str) -> Result<Account, String>{
    let mut db: Vec<Account> = match load_database(path) {
        Ok(acct) => acct,
        Err(err) => {
            return Err(err);
        }
    };

    // println!("database: {:?}", db);

    for account in db.iter_mut() {
        if account.account_number == generate_account_number(phone_no) {
            if verify_password(password, &account.user.password_hash) {
                account.user.status = ActiveStatus::LoggedIn;
                println!("Account logged in Successfully!");
                return Ok(account.clone());
            }
        }
    };

    Err("Invalid Phone Number or Passowrd".to_string())

}


fn add_product(price:f64, stock:i32, name:&str, category: Category, path:&str) {
    let mut db: Vec<Product> = match load_database(path) {
        Ok(product) => product,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };


    let new_product = Product::new(&name.to_string(), price, stock, category);

    db.push(new_product);

    let saved = match save_file(path, &db) {
        Ok(file) => file,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    if saved {
        println!("Product Added Successfully!")
    }else {
        println!("Product not Added Successfully!")
    }

    
}   


fn view_product(path:&str, serial:usize) -> Result<Product, String> {
    let db:Vec<Product> = match load_database(path) {
        Ok(product) => product,
        Err(err) => {
            return Err(err.to_string());
            
        }
    };

   
    println!("============================\nProduct Detials\n-----------------");
    match db.get(serial-1) {
        Some(product) => {
            Ok(product.clone())
        },
        None => return Err("Product not Found!".to_string())
    }

    // product_output(&db[serial-1], serial-1);
    
}

fn update_product(index:usize, path:&str) {
    let mut db: Vec<Product> = match load_database(path) {
        Ok(product) => product,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };


    match db.get_mut(index-1) {
        Some(product) => {
            println!("Add Product:");
                
                let product_name = user_input("Enter Product Name:");
                let price = user_input("Enter Price:");

                let price = match check_price(&price) {
                    Ok(num) => num,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                let stock = user_input("Enter the number of Stocks:");
                let stock = match check_stock(&stock) {
                    Ok(num) => num,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                println!("Select category:\n1. Food\n2. Electronics\n3. Fashion\n4. Grocery\n5. Computing");

                let cate = user_input("Select Category");
                let category = match get_category(&cate) {
                    Some(cat) => cat,
                    None => {
                        println!("Invalid Category");
                        return;
                    }
                };

                product.name = product_name.to_string();
                product.price = price;
                product.stock = stock;
                product.category = category
        }, 
        None => println!("Product not Found")
    }

    let saved = match save_file(path, &db) {
        Ok(file) => file,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    if saved {
        println!("Product Updated Successfully!")
    }else {
        println!("Product not Updated Successfully!")
    }

}


fn view_all_product(path:&str){
    let db:Vec<Product> = match load_database(path) {
        Ok(product) => product,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    if db.is_empty(){
        println!("There is no Available Product");
        return;
    }

    for (no, product) in db.iter().enumerate(){
        
        product_output(product, no);
    }
}

fn restock(stock:&str, serial_no:usize, path:&str) {
    let mut db: Vec<Product> = match load_database(path) {
        Ok(prod) => prod,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let new_stock = match check_stock(stock) {
        Ok(stock) => stock,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    match db.get_mut(serial_no-1) {
        Some(product) => {
            product.stock += new_stock;
            
        },
        None => println!("Product not Found!")
    }


    let saved = match save_file(path, &db) {
        Ok(done) => done,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };
    
    if saved {
        println!("{} Product Restocked Successfully", db[serial_no-1].name)
    }else {
        println!("{} Product Restocked not Successfully", db[serial_no-1].name)
    }
}

fn add_to_cart(user: &Account, quantity:i32, product:&Product, path:&str) {
    let mut cart_db: Vec<Cart> = match  load_database(path) {
        Ok(cart) => cart,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    if product.stock >= quantity {
        let cart = Cart::new(user, product, quantity);
        cart_db.push(cart);

        let saved = match save_file(path, &cart_db) {
            Ok(done) => done, 
            Err(err) => {
                println!("{}", err);
                return;
            }
        };

        if saved {
            println!("Product Successfully Added to the Cart")
        }else {
            println!("Product Not Added to the Cart")
        }
    }else {
        println!("Product Out of Stock")
    }


    
}


fn change_user_type(account:&Option<Account>, path:&str) -> Result<String, String> {
    let active_account = match verify_login(account.clone()) {
        Ok(account) => account,
        Err(err) => {
            return Err(err.to_string());            
        }
    };

    if active_account.user.user_type == UserType::Admin {
        let mut account_db: Vec<Account> = match load_database(path) {
            Ok(acct) => acct,
            Err(err) => {
                return Err(err.to_string());
            }
        };

        for (index, account) in account_db.iter().enumerate() {
            println!("User Accounts:\n{}. Full_name: {}\nEmail: {}\nUser Type: {}\nAccount Number: {}\n---------------------",
            index+1, 
            account.user.name.trim(),
            account.user.email.trim(),
            match account.user.user_type {
                UserType::Admin => "Admin",
                UserType::Guest => "Guest"
            },
            account.account_number
        )

        
        }

        let choosen_acct = user_input("Select a User:");

        let user_index:usize = match choosen_acct.trim().parse() {
            Ok(index) => index,
            Err(err) => {
                return Err("Invalid Input for the User Account".to_string());
                
            }
        };

        match account_db.get_mut(user_index-1) {
            Some(acct) => {
                acct.user.user_type = UserType::Admin
            },
            None => return Err("User Not Find".to_string())
        }

        let saved = match save_file(path, &account_db) {
            Ok(done) => done,
            Err(err) => {
                return Err(err.to_string());
            }
        };

        if saved {
            Ok("Account User Type Changed Successfully!".to_string())
        }else {
            Err("Change of User Type not Successful\n".to_string())
        }
    }else {
        return Err("Only Authorized User can perform this action!".to_string())
    }
    
}


fn delete_product(path:&str, serial:usize) {
    let mut prod_db:Vec<Product> = match load_database(path) {
        Ok(product) => product,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    match prod_db.get_mut(serial-1) {
        Some(prod) => {
            prod_db.remove(serial-1);
        },
        None => println!("Product not Found!")
    }


    let saved = match  save_file(path, &prod_db) {
        Ok(done) => done,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    if saved{
        println!("Product deleted Successfully")
    }else {
        println!("Product not deleted Successfully")
    }

}


fn view_all_cart(path:&str) {
    let cart_db: Vec<Cart> = match load_database(path) {
        Ok(cart) => cart,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };


    if !cart_db.is_empty(){
        println!("All Available Carts");
        for (index, cart) in cart_db.iter().enumerate() {
            cart_output(index, cart);
        }
    }else {
        println!("There is no Available Cart")
    }
}



















fn cart_output(index:usize, cart:&Cart){
    println!(
            "----------------------\n{} User Name: {}    -    User Email: {}\nProduct Name: {}  -  Product Price: N{:.2}\nQuantity: {}   -   Total: N{:.2}\n",
            index +1,
            cart.user.user.name.trim().to_uppercase(),
            cart.user.user.email.trim().to_lowercase(),
            cart.product.name.trim(),
            cart.product.price,
            cart.quantity,
            cart.quantity as f64 * cart.product.price
        )
}


fn hash_password(password: &str) -> String {

    
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


fn load_database<T: serde::de::DeserializeOwned> (path:&str) -> Result<Vec<T>, String> {
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


// fn validate_email(email:&str) -> bool {
//     email.validate_email()
// }


fn validate_email(email: &str) -> bool {
    let email = email.trim();

    email.contains('@')
        && email.contains('.')
        && !email.starts_with('@')
        && !email.ends_with('@')
}



fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }

    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err("Password must contain an uppercase letter".to_string());
    }

    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err("Password must contain a lowercase letter".to_string());
    }

    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err("Password must contain a digit".to_string());
    }

    Ok(())
}



fn verify_login(account:Option<Account>) -> Result<Account, String> {
    match account {
        Some(ref account) => {
            match account.user.status {
                ActiveStatus::LoggedIn => {
                    Ok(account.clone())
                }, 
                ActiveStatus::LoggedOut => Err("Please Login to Access this Page".to_string())
            }
        },
        None => Err("You can't Access This Page because you are not logged in".to_string())
    }
}


fn check_price(price:&str) -> Result<f64, String> {
    let price:f64 = match price.trim().parse() {
        Ok(num) => num,
        Err(err) => {
            return Err("Only Digits is accepted for price".to_string());
        }
    };

    if price <= 0.0 {
        return Err("Price cannot be less than or equall to zero".to_string());
    };

    Ok(price)
}


fn check_stock(stock:&str) -> Result<i32, String> {
    let stock:i32 = match stock.trim().parse() {
        Ok(num) => num,
        Err(err) => {
            return Err("Only Digits is accepted".to_string());
        }
    };

    if stock <= 0 {
        return Err("Number cannot be less than or equall to zero".to_string());
    };

    Ok(stock)
}


fn get_category(input:&str) -> Option<Category> {
    match input.trim().to_lowercase().as_str() {
        "1" => Some(Category::Food),
        "2" => Some(Category::Electronics),
        "3" => Some(Category::Fashion),
        "5" => Some(Category::Computing), 
        "4" => Some(Category::Grocery),
        _ =>  None
    }
}


fn product_output(product:&Product, serial_no:usize) {
    println!("{}.\nName: {}\nPrice: {}\nStock: {}\nCategory: {:?}\n", serial_no+1, product.name.trim(), product.price, product.stock, product.category);
}


fn single_product_commands(input: String, serial:usize, product: Product, cart_path: &str, product_path:&str, logged_in_account: &Option<Account>, ){
    match input.trim() {
        "1" => {
            let active_account = match verify_login(logged_in_account.clone()) {
                    Ok(account) => account,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

            
            if active_account.user.user_type == UserType::Admin {
                let stock = &user_input("Enter the number of Stocks:");
                restock(stock, serial, product_path);
            }else {
                println!("Only Admin can restock a product");
                // return;
            }
        },

        "2" => {
            let active_account = match verify_login(logged_in_account.clone()) {
                    Ok(account) => account,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

            
            if active_account.user.user_type == UserType::Admin {
                update_product(serial, product_path);
            }else {
                println!("Only Admin has the Permission to Perform the action")
            }
        },

        "3" => {
            let active_account = match verify_login(logged_in_account.clone()) {
                    Ok(account) => account,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

            let mut cart_db:Vec<Cart> = match load_database(cart_path) {
                Ok(cart) => cart,
                Err(err) => {
                    println!("{}", err);
                    return;
                }
            };

            let quantity = user_input("Enter Needed Quantity:");
            let quantity = match check_stock(&quantity) {
                Ok(num) => num,
                Err(err) => {
                    println!("{}", err);
                    return;
                }
            };


            if let Some(prod) = cart_db.iter_mut().find(|cart_product| cart_product.user.account_number == active_account.account_number && cart_product.product.name.trim().to_lowercase() == product.name.trim().to_lowercase()) {
                if quantity > product.stock {
                    println!("The requesting quantity is more than the Available stock")
                }else {
                    prod.quantity += quantity;
                }

            match save_file(cart_path, &cart_db) {
                    Ok(done) => done,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };
            } else {
                // println!("Not increased")
                add_to_cart(&active_account, quantity, &product, cart_path);
            }
        
        
        },

        "4" => {
            let active_account = match verify_login(logged_in_account.clone()) {
                    Ok(account) => account,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

            if active_account.user.user_type == UserType::Admin {
                delete_product(product_path, serial);
            }else {
                println!("Only Admin has the Permission to Perform the action")
            }
        }

        _ => println!("Invalid Input for Product Actions")
    }
}