#![allow(unused)]

use serde::{Deserialize, Serialize};
use std::{
    format, io::{self, BufReader, BufWriter}, println, vec,
};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand::{Rng, rngs::OsRng, seq::index};
use std::fs::File;
// use validator::{Validate, ValidateEmail};

// Used for the generated codes for different ids 
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct IDCodes{
    code: String
}

impl IDCodes {
    fn new(code:String) -> Self {
        Self { code: code }
    }
}


#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
enum Command {
    AddProduct,
    ViewAllProducts,
    ViewSingleProduct,
    ViewAllCart,
    ViewMyCart,
    ViewAllOrders,
    ViewMyOrders,
    CreateAccount,
    ChangeUserType,
    Login,
    FundAccount,
    Exit,
}

impl Command {
    fn menu_item_name(self) -> &'static str {
        match self {
            Command::AddProduct => "Add Product",
            Command::ViewAllProducts => "View All Products",
            Command::ViewSingleProduct => "View Single Product",
            Command::ViewAllCart => "View All Cart",
            Command::ViewMyCart => "View My Cart",
            Command::ViewAllOrders => "View All Orders",
            Command::ViewMyOrders => "View My Orders",
            Command::CreateAccount => "Create Account",
            Command::ChangeUserType => "Change User Type",
            Command::Login => "Login",
            Command::FundAccount => "Fund my Account",
            Command::Exit => "Exit",
        }
    }
}



const MENU_CONTENTS: &[Command] = &[
    Command::AddProduct,
    Command::ViewAllProducts,
    Command::ViewSingleProduct,
    Command::ViewAllCart,
    Command::ViewMyCart,
    Command::ViewAllOrders,
    Command::ViewMyOrders,
    Command::CreateAccount,
    Command::ChangeUserType,
    Command::Login,
    Command::FundAccount,
    Command::Exit,
];




// User Types
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
enum UserType {
    Admin,
    Guest,
}

// Activeness
#[derive(Serialize, PartialEq, Deserialize, Debug, Clone)]
enum ActiveStatus {
    LoggedIn,
    LoggedOut,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Category {
    Food,
    Electronics,
    Fashion,
    Grocery,
    Computing,
}

#[derive(Serialize, PartialEq, Deserialize, Debug, Clone)]
enum OrderStatus {
    Paid,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: String,
    name: String,
    address: String,
    email: String,
    user_type: UserType,
    password_hash: String,
    status: ActiveStatus,
}

impl User {
    fn new(name: String, address: String, code:&str, email: String, password: String) -> Self {
        let hashed = hash_password(password.as_str());

        Self {
            id: code.to_string(),
            name: name,
            address: address,
            email: email,
            user_type: UserType::Guest,
            password_hash: hashed,
            status: ActiveStatus::LoggedOut,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Account {
    user: User,
    account_number: String,
    balance: f64,
}

impl Account {
    fn new(user: &User, phone_no: &str) -> Self {
        let acc_no = generate_account_number(phone_no);

        Self {
            user: user.clone(),
            account_number: acc_no,
            balance: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Product {
    id: String,
    name: String,
    price: f64,
    stock: i32,
    category: Category,
}

impl Product {
    fn new(id: &str, name: &str, price: f64, stock: i32, category: Category) -> Self {
        Self {
            id: String::from(id),
            name: String::from(name),
            price,
            stock,
            category,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Cart {
    user_id: String,
    products: Vec<CartProduct>,
    active: bool,
}

impl Cart {
    fn new(user: &Account, product: Vec<CartProduct>, quantity: i32) -> Self {
        Self {
            user_id: user.user.id.clone(),
            products: product,
            active: true,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct CartProduct {
    product_id: String,
    quantity: i32
}


impl CartProduct {
    fn new(id: String, quantity: i32) -> Self{
        Self { product_id: id, quantity }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Order {
    order_id: String,
    user_id: String,
    products: Vec<CartProduct>,
    grand_total: f64,
    status: OrderStatus
}

impl Order {
    fn new(products: Vec<CartProduct>, grand_total:f64, user_id: &str, order_id: &str) -> Self {




        Self {
            order_id: order_id.to_string(),
            user_id: user_id.to_string(),
            products,
            grand_total,
            status: OrderStatus::Paid,
        }
    }
}

fn main() {
    println!("INVENTORY SYSTEM");
    let acct_path = "account.json";
    let product_path = "product.json";
    let cart_path = "cart.json";
    let order_path = "order.json";

    let mut logged_in_account: Option<Account> = None;
    loop {
        // println!(
        //     "Options: \n1. Add Product\n2. View All Products\n3. View Single Product\n4. View All Cart\n5. View My Cart\n6. View All Orders\n7. View My Orders\n8. Create Account\n9. Change User Type\n10. Login\n11. Fund my Account\n12. Exit"
        // );

        print_menu();

        let input = user_input("Please Select an Option:");

        match input.trim() {
            "1" => {
                println!("Add Product:");
                let active_account = match verify_login(logged_in_account.clone()) {
                    Ok(account) => {
                        let _product_db: Vec<Product> = match load_database(product_path) {
                            Ok(product) => product,
                            Err(err) => {
                                println!("{}", err);
                                continue;
                            }
                        };

                        

                        if account.user.user_type == UserType::Admin {
                            let product_name = user_input("Enter Product Name:");
                            let price = user_input("Enter Price:");
                            let stock = user_input("Enter the number of Stocks:");
                            
                            println!(
                                "Select category:\n1. Food\n2. Electronics\n3. Fashion\n4. Grocery\n5. Computing"
                            );
                            
                            let cate = user_input("Select Category");
                            match add_product(&price, &stock, &product_name, &cate, product_path) {
                                Ok(_) => println!("Product Added Successfully"),
                                Err(err) => {
                                    println!("{}", err);
                                    continue;
                                }
                            }
                            
                            
                        } else {
                            println!("Only Admin can perform this action")
                        }
                    },
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };

                
            }

            "2" => {
                println!("\nAll Available Product:");
                view_all_product(product_path);
            }

            "3" => {
                println!("\nAll Available Product:");
                view_all_product(product_path);

                let product_input = user_input("Select a Product:");
                let serial: usize = match product_input.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("Invalid Input");
                        continue;;
                    }
                };

                let product = match view_product(&product_path, serial) {
                    Ok(prod) => {
                        product_output(&prod, serial - 1);

                        println!(
                            "Product Options:\n1. Restock Product\n2. Update Product\n3. Add to Cart\n4. Delete Product"
                        );
                        let input = user_input("Select an Option:");

                        single_product_commands(
                            input,
                            serial,
                            prod,
                            cart_path,
                            product_path,
                            &logged_in_account,
                        );
                    },
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };

                
            }

            "4" => {
                let active_account = match verify_login(logged_in_account.clone()) {
                    Ok(account) => account,
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };

                if active_account.user.user_type == UserType::Admin {
                    match view_all_cart(acct_path, product_path, cart_path) {
                        Ok(_) => {},
                        Err(err) => println!("{}", err)
                    }
                } else {
                    println!("Only Admin can perform this action")
                }
            }

            "5" => {
                let account = match verify_login(logged_in_account.clone()) {
                    Ok(account) => account,
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };

                match view_my_cart(cart_path, product_path, acct_path, &account) {
                    Ok(_) => {},
                    Err(err) => {
                        println!("{}", err)
                    }
                }

                println!("Cart Options:\n1. Order Now\n2. Delete Item\n3. Clear Cart");
                let input = user_input("Select an Option:");

                // match input.trim() {
                //     "1" => {
                //         // Placing Order
                //         // order_now(&active_account, carts, acct_path, cart_path, order_path);
                //     }

                //     "2" => {
                //         // Deleting Item from my cart
                //         delete_item_from_my_cart(&active_account, &mut carts, cart_path);
                //     }

                //     "3" => {
                //         // Clearing Cart
                //         clear_my_cart(&active_account, &mut carts, cart_path);
                //     }

                //     _ => println!("Invalid Input"),
                // }
            }

            "6" => {
                let active_account = match verify_login(logged_in_account.clone()) {
                    Ok(account) => account,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                if active_account.user.user_type != UserType::Admin {
                    println!("Only Admin can perform this Action");
                    return;
                }

                view_all_order(order_path);

                let input = user_input("Select an Order to Update the Status");

                let index: usize = match input.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("Invalid input");
                        return;
                    }
                };

                update_order_status(order_path, index);
            }

            "7" => {
                // Viewing my orders

                let active_account = match verify_login(logged_in_account.clone()) {
                    Ok(account) => account,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };
                view_my_orders(order_path, &active_account);
            }

            "8" => {
                let full_name = user_input("Enter Full Name:");
                let email = user_input("Enter Your Email:");
                
                let address = user_input("Enter your Full Address:");

                let password = user_input("Enter Account Password");
                let phone_no = user_input("Enter Phone Number");
                let path = acct_path;


                match create_account(&full_name, &email, &address, &password, &phone_no, path){
                            Ok(_) => println!("Account Created Successfully"),
                            Err(err) => println!("{}", err)
                        }
                

                

            }

            "9" => {
                match change_user_type(&logged_in_account, acct_path) {
                    Ok(done) => {
                        println!("{}", done)
                    }
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };
            }

            "10" => {
                println!("Account Login");
                let phone_no = user_input("Enter Your Phone Number:");
                let phone_no = match check_phone_number(&phone_no) {
                    Ok(phone) => {
                        let password = user_input("Enter Password");

                        logged_in_account = match login(&phone_no, &password, acct_path) {
                            Ok(acct) => Some(acct),
                            Err(err) => {
                                println!("{}", err);
                                continue;
                            }
                        };
                    },
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };
                

                // println!("Account acive: {:#?}", logged_in_account);
            }

            "11" => {
                // Funding Account codes
                let active_account = match verify_login(logged_in_account.clone()) {
                    Ok(account) => account,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                let amount = user_input("Enter Amount to Fund:");
                let new_amount = match check_price(&amount) {
                    Ok(amount) => amount,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                fund_my_account(&active_account, new_amount, acct_path);
            }

            "12" => {
                println!("Exiting.....");
                break;
            }

            _ => {
                println!("Invalid Input")
            }
        }
    }
}

fn create_account(
    name: &str,
    email: &str,
    address: &str,
    password: &str,
    phone_no: &str,
    path: &str,
) -> Result<(), String>{
    let mut db: Vec<Account> = match load_database(path) {
        Ok(account) => account,
        Err(err) => {
            return Err(err.to_string());
        }
    };

    // Email validation
    if !validate_email(&email) {
        return Err("Invalid Email".to_string())
    }


    // Password validation
    match validate_password(&password) {
        Ok(pass) => pass,
        Err(err) => {
            return Err(err.to_string())
        }
    };


    match check_phone_number(&phone_no) {
        Ok(_) => {},
        Err(err) => {
            return Err(err);
        }
    };

    let code = match generate_codes("UCIS") {
        Ok(code) => code,
        Err(err) => {
            return Err(err)
        }
    };


    let exists= db.iter().any(|acct| acct.account_number == generate_account_number(&phone_no));

    if exists{
        // println!("Account Already Existed!");
        return Err("Account Already Existed!".to_string());
    }

    let user = User::new(name.to_string(), address.to_string(), &code, email.to_string(), password.to_string());
    let acct = Account::new(&user, &phone_no);

    db.push(acct);

    match save_file(path, &db) {
        Ok(_) => Ok(()),
        Err(_) => {
            Err("Account not Created Successfully!".to_string())
        }
    };

    Ok(())
}



fn login(phone_no: &str, password: &str, path: &str) -> Result<Account, String> {
    let mut db: Vec<Account> = match load_database(path) {
        Ok(acct) => acct,
        Err(err) => {
            return Err(err);
        }
    };

    // println!("database: {:?}", db);

    // if 

    for account in db.iter_mut() {
        if account.account_number == generate_account_number(phone_no) {
            if verify_password(password, &account.user.password_hash) {
                account.user.status = ActiveStatus::LoggedIn;
                println!("Account logged in Successfully!");
                return Ok(account.clone());
            }
        }
    }

    Err("Invalid Phone Number or Passowrd".to_string())
}

fn add_product(price: &str, stock: &str, name: &str, category: &str, path: &str) -> Result<(), String> {
    
    let mut db: Vec<Product> = match load_database(path) {
        Ok(product) => product,
        Err(err) => {
            return Err(err.to_string())
        }
    };

    let stock = match check_stock(&stock) {
        Ok(num) => num,
        Err(err) => {
            return Err(err)
        }
    };

    let price = match check_price(&price) {
        Ok(num) => num,
        Err(err) => {
            return Err(err)
        }
    };

    let category = match get_category(category) {
        Some(cat) => cat,
        None => {
            return Err("Invalid Category".to_string())
        }
    };
    
    match generate_codes("PCIS"){
        Ok(code ) => {
            let new_product = Product::new(&code, &name.to_string(), price, stock, category);

            db.push(new_product);

            match save_file(path, &db) {
                Ok(_) => return Ok(()),
                Err(err) => {
                    return Err(err)
                }
            }
        },
        Err(err) => {
            return Err(err);
            
        }
    };

    
    
    
}

fn view_product(path: &str, serial: usize) -> Result<Product, String> {
    let db: Vec<Product> = match load_database(path) {
        Ok(product) => product,
        Err(err) => {
            return Err(err);
        }
    };

    println!("============================\nProduct Detials\n-----------------");
    match db.get(serial - 1) {
        Some(product) => Ok(product.clone()),
        None => Err("Product not Found!".to_string()),
    }

    // product_output(&db[serial-1], serial-1);
}

fn update_product(index: usize, path: &str) -> Result<(), String>{
    let mut db: Vec<Product> = match load_database(path) {
        Ok(product) => product,
        Err(err) => {
            return Err(err);
        }
    };

    match db.get_mut(index - 1) {
        Some(product) => {
            println!("Add Product:");

            let product_name = user_input("Enter Product Name:");
            let price = user_input("Enter Price:");

            let price = match check_price(&price) {
                Ok(num) => num,
                Err(err) => {
                    return Err(err);
                }
            };

            let stock = user_input("Enter the number of Stocks:");
            let stock = match check_stock(&stock) {
                Ok(num) => num,
                Err(err) => {
                    return Err(err);
                }
            };

            println!(
                "Select category:\n1. Food\n2. Electronics\n3. Fashion\n4. Grocery\n5. Computing"
            );

            let cate = user_input("Select Category");
            let category = match get_category(&cate) {
                Some(cat) => cat,
                None => {
                    return Err("Invalid Category".to_string());
                    
                }
            };

            product.name = product_name.to_string();
            product.price = price;
            product.stock = stock;
            product.category = category;


            match save_file(path, &db) {
                Ok(_) => return Ok(()),
                Err(err) => {
                    return Err(err);
                }
            };

            // return Ok("Product Update Done".to_string())
        }
        None => return Err("Product not Found".to_string()),
    }

    

    
}

fn view_all_product(path: &str) {
    let db: Vec<Product> = match load_database(path) {
        Ok(product) => product,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    if db.is_empty() {
        println!("There is no Available Product");
        return;
    }

    for (no, product) in db.iter().enumerate() {
        product_output(product, no);
    }
}

fn restock(stock: &str, serial_no: usize, path: &str) -> Result<String, String> {
    let mut db: Vec<Product> = match load_database(path) {
        Ok(prod) => prod,
        Err(err) => {
            return Err(err);
            
        }
    };

    let new_stock = match check_stock(stock) {
        Ok(stock) => stock,
        Err(err) => {
            return Err(err);
            
        }
    };

    match db.get_mut(serial_no - 1) {
        Some(product) => {
            product.stock += new_stock;
        }
        None => return Err("Product not Found!".to_string()),
    }

    match save_file(path, &db) {
        Ok(_) => {
            let response = format!("{} Product Restocked Successfully", db[serial_no - 1].name);
            return Ok(response)
        },
        Err(err) => {
            return Err(err);
            
        }
    };

    Ok("Restocked".to_string())
}

fn add_to_cart(user: &Account, quantity: i32, product: &Product, path: &str)-> Result<(), String> {
    let mut cart_db: Vec<Cart> = match load_database(path) {
        Ok(cart) => cart,
        Err(err) => {
            return Err(err);
        }
    };

    let mut cart_product:Vec<CartProduct> = Vec::new();

    let new_product = CartProduct::new(product.id.clone(), quantity);
    cart_product.push(new_product);

    if product.stock >= quantity {
        let cart = Cart::new(user, cart_product, quantity);
        cart_db.push(cart);

        match save_file(path, &cart_db) {
            Ok(_) => return Ok(()),
            Err(err) => {
                return Err(err);
            }
        };
    } else {
        return Err("Product Out of Stock".to_string());
    }
}

fn change_user_type(account: &Option<Account>, path: &str) -> Result<String, String> {
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
            println!(
                "User Accounts:\n{}. Full_name: {}\nEmail: {}\nUser Type: {}\nAccount Number: {}\n---------------------",
                index + 1,
                account.user.name.trim(),
                account.user.email.trim(),
                match account.user.user_type {
                    UserType::Admin => "Admin",
                    UserType::Guest => "Guest",
                },
                account.account_number
            )
        }

        let choosen_acct = user_input("Select a User:");

        let user_index: usize = match choosen_acct.trim().parse() {
            Ok(index) => index,
            Err(_) => {
                return Err("Invalid Input for the User Account".to_string());
            }
        };

        match account_db.get_mut(user_index - 1) {
            Some(acct) => acct.user.user_type = UserType::Admin,
            None => return Err("User Not Found".to_string()),
        }

        match save_file(path, &account_db) {
            Ok(_) => return Ok("Account User Type Changed Successfully!".to_string()),
            Err(err) => {

                let error = format!("Change of User Type not Successful\nError: {}", err);
                return Err(error.to_string())
            }
        };

    }
    return Err("Only Authorized User can perform this action!".to_string());
}

fn delete_product(path: &str, serial: usize)-> Result<(), String> {
    let mut prod_db: Vec<Product> = match load_database(path) {
        Ok(product) => product,
        Err(err) => {
            return Err(err);
        }
    };

    match prod_db.get_mut(serial - 1) {
        Some(_prod) => {
            prod_db.remove(serial - 1);

            match save_file(path, &prod_db) {
                Ok(_) => return Ok(()),
                Err(err) => {
                    return Err(err);
                }
            };
        }
        None => return Err("Product not Found!".to_string()),
    }

    

    
}

fn view_all_cart(acct_path: &str, product_path: &str, cart_path: &str) -> Result<(), String>{
    let cart_db: Vec<Cart> = match load_database(cart_path) {
        Ok(cart) => cart,
        Err(err) => {
            return Err(err);
        }
    };

    let prod_db: Vec<Product> = match load_database(product_path) {
        Ok(cart) => cart,
        Err(err) => {
            return Err(err);
        }
    };

    let acct_db: Vec<Account> = match load_database(acct_path) {
        Ok(cart) => cart,
        Err(err) => {
            return Err(err);
        }
    };




    if !cart_db.is_empty() {
        println!("All Available Carts");
        for (index, cart) in cart_db.iter().enumerate() {
            if let Some(cart_account) = acct_db.iter().find(|account| account.user.id == cart.user_id) {
                println!("----------------------\n{}. User Name: {}    -    User Email: {}\n=====================\n Products\n=======================", index+1, cart_account.user.name, cart_account.user.email.to_lowercase());

                for (prod_index, cart_product) in cart.products.iter().enumerate(){
                    let Some(user_prod) = prod_db.iter().find(|p| p.id == cart_product.product_id) else {
                        continue;
                    };
                        println!(
                                    "----------------------\n{} Product Name: {}  -  Product Price: N{:.2}\nQuantity: {}   -   Total: N{:.2}\n",
                                    prod_index + 1,
                                    user_prod.name.trim(),
                                    user_prod.price,
                                    cart_product.quantity,
                                    cart_product.quantity as f64 * user_prod.price
                                );
                    }
                }
                println!("++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++")
            }
        }

        Ok(())
    } 





fn view_my_cart(cart_path: &str, product_path: &str, acct_path: &str, account: &Account) -> Result<(), String> {
    let cart_db: Vec<Cart> = match load_database(cart_path) {
        Ok(cart) => cart,
        Err(err) => {
            return Err(err);
        }
    };

    let prod_db: Vec<Product> = match load_database(product_path) {
        Ok(cart) => cart,
        Err(err) => {
            return Err(err);
        }
    };

    let acct_db: Vec<Account> = match load_database(acct_path) {
        Ok(cart) => cart,
        Err(err) => {
            return Err(err);
        }
    };




    if !cart_db.is_empty() {
        println!("Your Available Carts");
        for cart in cart_db.iter() {
            if let Some(cart_account) = acct_db.iter().find(|account| account.user.id == account.user.id) {
                println!("----------------------\nUser Name: {}    -    User Email: {}\n=====================\n Products\n=======================", cart_account.user.name, cart_account.user.email.to_lowercase());

                for (prod_index, cart_product) in cart.products.iter().enumerate(){
                    let Some(user_prod) = prod_db.iter().find(|p| p.id == cart_product.product_id) else {
                        continue;
                    };
                        println!(
                                    "----------------------\n{} Product Name: {}  -  Product Price: N{:.2}\nQuantity: {}   -   Total: N{:.2}\n",
                                    prod_index + 1,
                                    user_prod.name.trim(),
                                    user_prod.price,
                                    cart_product.quantity,
                                    cart_product.quantity as f64 * user_prod.price
                                );
                    }
                }
                println!("++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++")
            }
        }

        Ok(())
}

fn fund_my_account(account: &Account, amount: f64, path: &str) {
    let mut acct_db: Vec<Account> = match load_database(path) {
        Ok(account) => account,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let acct_index: usize = match acct_db
        .iter()
        .position(|c| c.account_number == account.account_number)
    {
        Some(index) => index,
        None => {
            println!("Account not Found!");
            return;
        }
    };

    acct_db[acct_index].balance += amount;

    match save_file(path, &acct_db) {
        Ok(_done) => {
            println!("Account funded Successfully");
        }
        Err(err) => {
            println!("{err}");
            return;
        }
    };
}

fn order_now(
    account: &Account,
    acct_path: &str,
    cart_path: &str,
    product_path: &str,
    order_path: &str,
) -> Result<(), String>{
    let mut cart_db: Vec<Cart> = match load_database(cart_path) {
        Ok(cart) => cart,
        Err(err) => {
            return Err(err);
        }
    };

    let mut account_db: Vec<Account> = match load_database(acct_path) {
        Ok(acct) => acct,
        Err(err) => {
            return Err(err);
        }
    };

    let mut prod_db: Vec<Product> = match load_database(product_path) {
        Ok(product) => product,
        Err(err) => {
            return Err(err);
        }
    };

    let mut order_db: Vec<Order> = match load_database(order_path) {
        Ok(order) => order,
        Err(err) => {
            return Err(err);
        }
    };

    let mut grand_total = 0.0;


    for cart in cart_db.iter(){
        if let Some(user) = account_db.iter().find(|u| u.user.id == cart.user_id) {
            for product in cart.products.iter(){
                if let Some(cart_prod) = prod_db.iter().find(|prod| prod.id == product.product_id){
                    grand_total += product.quantity as f64 * cart_prod.price
                }
            }

            let order_id = match generate_codes("ORD"){
                Ok(id) => id,
                Err(err) => return Err(err)
            };

            let new_order = Order::new(cart.products.clone(), grand_total, &user.user.id, &order_id);

            if user.balance >= grand_total {
                order_db.push(new_order);

                match save_file(order_path, &order_db) {
                    Ok(_) => {
                        for cart_product in cart.products.iter(){
                            let prod_index = match prod_db.iter().position(|prod| prod.id ==cart_product.product_id) {
                                Some(index) => index,
                                None => {
                                    return Err("Product not found".to_string());
                                }
                            };

                            match prod_db.get_mut(prod_index) {
                                Some(the_prod) => {
                                    the_prod.stock -= cart_product.quantity
                                },
                                None => {
                                    return Err("System Error!".to_string());
                                }
                            }
                        }

                        match save_file(product_path, &prod_db) {
                            Ok(_) => {},
                            Err(err) => return Err(err)
                        }
                    },
                    Err(err) => return Err(err)
                }
            }else {
                return Err("Insufficient balance to complete this order".to_string())
            }
            
        } else {
            return Err("Orders not successful".to_string());
        }

    }

    Ok(())

    
}

fn delete_item_from_my_cart(account: &Account, carts: &mut Vec<Cart>, cart_path: &str) {
    let mut cart_db: Vec<Cart> = match load_database(cart_path) {
        Ok(cart) => cart,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let mut dindex: usize = 0;

    for (index, cart) in carts.iter().enumerate() {
        cart_output(index, cart);
    }

    let input = user_input("Select an Item to delete:");
    let index: usize = match input.trim().parse() {
        Ok(num) => num,
        Err(_err) => {
            println!("Invalid Input");
            return;
        }
    };

    match carts.get_mut(index - 1) {
        Some(my_cart) => {
            dindex = match cart_db.iter().position(|c| { (c.user_id == account.user.id) }) {
                Some(index) => index,
                None => {
                    println!("Cart Item not Found!");
                    return;
                }
            };
        }
        None => println!(""),
    }

    match cart_db.get_mut(dindex) {
        Some(_cart) => {
            cart_db.remove(dindex);

            match save_file(cart_path, &cart_db) {
                Ok(_done) => println!("Item removed Successfully"),
                Err(_err) => println!("Item Not Removed!"),
            }
        }
        None => println!("Item not found!"),
    }
}

fn clear_my_cart(account: &Account, carts: &mut Vec<Cart>, cart_path: &str) {
    let mut cart_db: Vec<Cart> = match load_database(cart_path) {
        Ok(cart) => cart,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    if carts.is_empty() {
        println!("You have no Item in your cart");
        return;
    }

    for my_cart in carts.iter_mut() {
        let index = match cart_db.iter().position(|c| {
            (c.user_id == account.user.id)
                
        }) {
            Some(index) => index,
            None => {
                println!("Item not found");
                return;
            }
        };
        match cart_db.get_mut(index) {
            Some(_cart) => {
                cart_db.remove(index);
            }
            None => {
                println!("Cart Not Cleared");
                return;
            }
        }
    }

    match save_file(cart_path, &cart_db) {
        Ok(_) => println!("Cart cleared Successfully"),
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    
}

fn view_all_order(path: &str) {
    let order_db: Vec<Order> = match load_database(path) {
        Ok(cart) => cart,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    if !order_db.is_empty() {
        println!("All Available Order(s)");
        for (index, order) in order_db.iter().enumerate() {
            order_output(index, order);
        }
    } else {
        println!("There is no Available Order")
    }
}

fn update_order_status(path: &str, index: usize) {
    let mut order_db: Vec<Order> = match load_database(path) {
        Ok(order) => order,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    println!("Options: \n1. Processing\n2. Shipped\n3. Delivered\n4. Cancelled");
    let input = user_input("Select an Option:");

    let new_status = match get_order_status(&input) {
        Some(status) => status,
        None => {
            println!("Invalid Status Selection");
            return;
        }
    };

    match order_db.get_mut(index - 1) {
        Some(order) => {
            order.status = new_status;

            match save_file(path, &order_db) {
                Ok(_done) => {
                    println!("Order Updated Successfully");
                }
                Err(err) => {
                    println!("{}", err)
                }
            }
        }
        None => {
            println!("Invalid Selection of Order");
            return;
        }
    }
}

fn view_my_orders(path: &str, account: &Account) {
    let order_db: Vec<Order> = match load_database(path) {
        Ok(order) => order,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    for (index, order) in order_db.iter().enumerate() {
        if order.user.account_number == account.account_number {
            order_output(index, order);
        }
    }

    let input = user_input("Select an Order to Check Status:");

    view_my_single_order(&input, path)
}

fn view_my_single_order(input: &str, path: &str) {
    let order_db: Vec<Order> = match load_database(path) {
        Ok(order) => order,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let index: usize = match input.trim().parse() {
        Ok(num) => num,
        Err(_err) => {
            println!("Invalid Input Selection");
            return;
        }
    };

    match order_db.get(index - 1) {
        Some(order) => {
            order_output(index, order);
        }
        None => {
            println!("Order not found!")
        }
    }
}

fn order_output(index: usize, order: &Order) {
    println!(
        "----------------------\n{}. User Name: {}    -    User Email: {}\nOrders: {:#?}\nOrder Status: {:?}   -   Grand Total: N{:.2}\n",
        index + 1,
        order.user.user.name.trim().to_uppercase(),
        order.user.user.email.trim().to_lowercase(),
        order.carts,
        order.status,
        order.grand_total
    )
}

fn cart_output(index: usize, cart: &Cart) -> Result<(), String>{
    let product_path = "product.json";
    let acct_path = "account.json";

    let acct_db:Vec<Account> = match load_database(acct_path) {
        Ok(acct) => acct,
        Err(err) => {
            return Err(err);
        }
    };

    let prod_db:Vec<Product> = match load_database(product_path) {
        Ok(acct) => acct,
        Err(err) => {
            return Err(err);
        }
    };
    
    let user_index = match acct_db.iter().position(|a| a.user.id == cart.user_id){
        Some(index) => index,
        None => {
            return Err("User's Cart is Empty!".to_string());
        }
    };

    // let prod_index = match prod_db.iter().position(|a| a.id == cart.product_id){
    //     Some(index) => index,
    //     None => {
    //         return Err("User's Cart is Empty!".to_string());
    //     }
    // };

    Ok(())

    // println!(
    //     "----------------------\n{}. User Name: {}    -    User Email: {}\nProduct Name: {}  -  Product Price: N{:.2}\nQuantity: {}   -   Total: N{:.2}\n",
    //     index + 1,
    //     cart.user.user.name.trim().to_uppercase(),
    //     cart.user.user.email.trim().to_lowercase(),
    //     cart.product.name.trim(),
    //     cart.product.price,
    //     cart.quantity,
    //     cart.quantity as f64 * cart.product.price
    // )
}

fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(OsRng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

fn generate_account_number(phone_no: &str) -> String {
    let phone_no = phone_no.trim();
    let number = phone_no.len() - 10;
    let new_number = &phone_no[number..];
    String::from(new_number)
}

fn check_phone_number(phone_no: &str) -> Result<(), String> {
    let phone_no = phone_no.trim();

    if !phone_no.chars().all(|c| c.is_ascii_digit()) {
        return Err("Phone number should contain only digits".to_string());
    }

    // if !phone_no.starts_with("0") {
    //     return Err("Phone Number must start with zeror".to_string());
    // }

    if phone_no.len() < 11 {
        return Err("Phone number must be 11 digits".to_string());
    }

    Ok(())
}

fn user_input(option: &str) -> String {
    println!("{}", option);

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("No Input Found!");

    input
}

fn load_database<T: serde::de::DeserializeOwned>(path: &str) -> Result<Vec<T>, String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_err) => {
            // let _error = format!("Open File Error: {}", err);
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

fn save_file<T: Serialize>(path: &str, database: &Vec<T>) -> Result<(), String> {
    let file = match File::create(path) {
        Ok(file) => file,
        Err(err) => {
            let error = format!("Failed with the Error: {}", err);
            return Err(error.to_string());
        }
    };

    let writer = BufWriter::new(file);

    match serde_json::to_writer_pretty(writer, &database) {
        Ok(_) => Ok(()),
        Err(_) => Err("File not saved!".to_string()),
    }
}

// fn validate_email(email:&str) -> bool {
//     email.validate_email()_
// }

fn validate_email(email: &str) -> bool {
    let email = email.trim();

    email.contains('@') && email.contains('.') && !email.starts_with('@') && !email.ends_with('@')
}

fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 4 {
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

fn verify_login(account: Option<Account>) -> Result<Account, String> {
    match account {
        Some(ref account) => match account.user.status {
            ActiveStatus::LoggedIn => Ok(account.clone()),
            ActiveStatus::LoggedOut => Err("Please Login to Access this Page".to_string()),
        },
        None => Err("You can't Access This Page because you are not logged in".to_string()),
    }
}

fn check_price(price: &str) -> Result<f64, String> {
    let price: f64 = match price.trim().parse() {
        Ok(num) => num,
        Err(_err) => {
            return Err("Only Digits is accepted for price".to_string());
        }
    };

    if price <= 0.0 {
        return Err("Price cannot be less than or equall to zero".to_string());
    };

    Ok(price)
}

fn check_stock(stock: &str) -> Result<i32, String> {
    let stock: i32 = match stock.trim().parse() {
        Ok(num) => num,
        Err(_err) => {
            return Err("Only Digits is accepted".to_string());
        }
    };

    if stock <= 0 {
        return Err("Number cannot be less than or equall to zero".to_string());
    };

    Ok(stock)
}

fn get_order_status(input: &str) -> Option<OrderStatus> {
    match input.trim() {
        "1" => Some(OrderStatus::Processing),
        "2" => Some(OrderStatus::Shipped),
        "3" => Some(OrderStatus::Delivered),
        "4" => Some(OrderStatus::Cancelled),
        _ => None,
    }
}

fn get_category(input: &str) -> Option<Category> {
    match input.trim().to_lowercase().as_str() {
        "1" => Some(Category::Food),
        "2" => Some(Category::Electronics),
        "3" => Some(Category::Fashion),
        "5" => Some(Category::Computing),
        "4" => Some(Category::Grocery),
        _ => None,
    }
}


fn print_menu() {
    println!("Options:");

    for (index, command) in MENU_CONTENTS.iter().enumerate() {
        println!("{}. {}", index + 1, command.menu_item_name());
    }
}


fn product_output(product: &Product, serial_no: usize) {
    println!(
        "{}.\nName: {}\nPrice: {}\nStock: {}\nCategory: {:?}\n",
        serial_no + 1,
        product.name.trim(),
        product.price,
        product.stock,
        product.category
    );
}

fn single_product_commands(
    input: String,
    serial: usize,
    product: Product,
    cart_path: &str,
    product_path: &str,
    logged_in_account: &Option<Account>,
) -> Result<(), String> {

    let active_account = match verify_login(logged_in_account.clone()) {
            Ok(account) => account,
            Err(err) => {
                return Err(err);
            }
        };

    match input.trim() {
        "1" => {
            
            if active_account.user.user_type == UserType::Admin {
                let stock = &user_input("Enter the number of Stocks:");
                match restock(stock, serial, product_path) {
                    Ok(res) => return Ok(()),
                    Err(err) => {
                        return Err(err);
                    }
                };
            } else {
                Err("Only Admin can restock a product".to_string())
                // return;
            }
        }

        "2" => {
            // UPDATING A SINGLE PRODUCT 

            if active_account.user.user_type == UserType::Admin {
                match update_product(serial, product_path) {
                    Ok(_) => {
                        println!("Product Updated Successfully");
                        return Ok(())
                    },
                    Err(err) => return Err(err)
                };
            } else {
                Err("Only Admin has the Permission to Perform the action".to_string())
            }
        }

        "3" => {
            
            let mut cart_db: Vec<Cart> = match load_database(cart_path) {
                Ok(cart) => cart,
                Err(err) => {
                    return Err(err);
                }
            };

            let quantity = user_input("Enter Needed Quantity:");
            let quantity = match check_stock(&quantity) {
                Ok(num) => num,
                Err(err) => {
                    return Err(err);
                }
            };

            if let Some(prod) = cart_db.iter_mut().find(|cart_product| { cart_product.user_id == active_account.user.id }) {
                if quantity > product.stock {
                    return Err("The requesting quantity is more than the Available stock".to_string())
                } else {
                    
                    if let Some(new_prod) = prod.products.iter_mut().find(|p| p.product_id == product.id) {
                        new_prod.quantity += quantity;
                    }else {
                        let mut cart_product:Vec<CartProduct> = Vec::new();

                        let new_product = CartProduct::new(product.id, quantity);
                        prod.products.push(new_product);

                        println!("Product Added to cart!")
                    }
                    
                    match save_file(cart_path, &cart_db) {
                    Ok(_) => return Ok(()),
                    Err(err) => {
                        return Err(err);
                    }
                };

                }

                
            } else {
                // println!("Not increased")
                match add_to_cart(&active_account, quantity, &product, cart_path) {
                    Ok(_) => {
                        println!("Added to Cart Successfully");
                        return Ok(())
                    },
                    Err(err) => {
                        return Err(err);
                    }
                };
                
            }
        }

        "4" => {
            
            if active_account.user.user_type == UserType::Admin {
                match delete_product(product_path, serial) {
                    Ok(_) => return Ok(()),
                    Err(err) => {
                        return Err(err);
                    }
                };
                
            } else {
                return Err("Only Admin has the Permission to Perform the action".to_string())
            }
        }

        _ => return Err("Invalid Input for Product Actions".to_string()),
    }
}


fn generate_codes(initial: &str) -> Result<String, String> {
    const CODE_PATH: &str = "code_ids.json";

    let mut codes: Vec<IDCodes> = match load_database(CODE_PATH) {
        Ok(code) => code,
        Err(err) => {
            return Err(err);
        }
    };

  
    loop {
        let number = rand::thread_rng().gen_range(1000..=9999);

        let new_code = format!("{}{}", initial.to_uppercase(), number);

        let exists = codes.iter().any(|code| (code.code == new_code));

        if !exists {
            codes.push(IDCodes::new(new_code.clone()));

            match save_file(CODE_PATH, &codes) {
                Ok(_) => return Ok(new_code),
                Err(err) => return Err(err),
            }
        }
    }


}