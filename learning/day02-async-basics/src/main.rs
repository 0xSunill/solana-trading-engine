// use std::sync::Arc;

// #[tokio::main]
// async fn main() {
//     let message = Arc::new(String::from("Hello"));

//     // create 3 Arc clones
//     let greeting_a = Arc::clone(&message);
//     let greeting_b = Arc::clone(&message);
//     let greeting_c = Arc::clone(&message);

//     // spawn task A
//     let handler_a = tokio::spawn(async move {
//         println!("greeing is {}",greeting_a);
//     });
//     // spawn task B
//     let handler_b = tokio::spawn(async move {
//         println!("greeing is {}",greeting_b);
//     });
//     // spawn task C
//     let handler_c = tokio::spawn(async move {
//         println!("greeing is {}",greeting_c);
//     });

//     // await all 3
    
//     handler_a.await.unwrap();
//     handler_b.await.unwrap();
//     handler_c.await.unwrap();
// }


// use std::sync::Arc;
// use tokio::sync::Mutex;

// #[derive(Debug)]
// struct Portfolio {
//     balance: f64,
// }


// #[tokio::main]
// async fn main() {
//     let portfolio = Arc::new(Mutex::new(
//         Portfolio{
//             balance:1000.0,
//         }
//     ));


//     let portfolio_a = Arc::clone(&portfolio);
//     let portfolio_b = Arc::clone(&portfolio);

//     let handle_a = tokio::spawn(async move{
//         let mut portfolio = portfolio_a.lock().await;
//         portfolio.balance += 500.0;
//         println!("Updated the task 1 balance")
//     });

//       let handle_b = tokio::spawn(async move{
//         let mut portfolio = portfolio_b.lock().await;
//         portfolio.balance -= 1000.0;
//         println!("Updated the task 2 balance")
//     });

    
//     handle_a.await.unwrap();
//     handle_b.await.unwrap();
//     let portfolio = portfolio.lock().await;
//     println!("final balance is {:?}",*portfolio);

// }





// use std::sync::Arc;
// use tokio::sync::Mutex;

// #[derive(Debug)]
// struct Counter {
//     value: u64,
// }


// #[tokio::main]
// async fn main() {
//     let counter = Arc::new(Mutex::new(
//         Counter{
//            value:0,
//         }
//     ));


//     let counter_a = Arc::clone(&counter);
//     let counter_b = Arc::clone(&counter);
//     let counter_c = Arc::clone(&counter);

    
//     let handle_a = tokio::spawn(async move{
//         for _ in 1..=10{
//         let mut counter = counter_a.lock().await;
//         counter.value += 1;
//         println!("Updated the counter 1 value");
//         };
//     });

//      let handle_b = tokio::spawn(async move{
//         for _ in 1..=10{
//         let mut counter = counter_b.lock().await;
//         counter.value += 1;
//         println!("Updated the counter 2 value");
//         };

//     }); 

//     let handle_c = tokio::spawn(async move{
        
//         for _ in 1..=10{
//         let mut counter = counter_c.lock().await;
//         counter.value += 1;
//         println!("Updated the counter 3 value");
//         };
//     });

    
//     handle_a.await.unwrap();
//     handle_b.await.unwrap();
//     handle_c.await.unwrap();
    
//     let counter = counter.lock().await;
//     println!("final balance is {:?}",*counter);

// }







use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
struct Counter {
    value: u64,
}

#[tokio::main]
async fn main() {
    let counter = Arc::new(RwLock::new(
        Counter{
           value:200,
        }
    ));


    let counter_a = Arc::clone(&counter);
    let counter_b = Arc::clone(&counter);
    let counter_c = Arc::clone(&counter);
  let counter_d = Arc::clone(&counter);
    
    let handle_a = tokio::spawn(async move{
        
        let counter = counter_a.read().await;
        println!("reading {}", counter.value);
    
    });

     let handle_b = tokio::spawn(async move{
    
        let mut counter = counter_b.write().await;
        counter.value += 1;
        println!("Updated the value");
    }); 

    let handle_c = tokio::spawn(async move{
        
      
        let counter = counter_c.read().await;
        println!("reading {}", counter.value);
    });
      let handle_d = tokio::spawn(async move{
        
      
        let counter = counter_d.read().await;
        println!("reading {}", counter.value);
    });

    
    handle_a.await.unwrap();
    handle_b.await.unwrap();
    handle_c.await.unwrap();
    handle_d.await.unwrap();
    
    let counter = counter.read().await;
    println!("final balance is {:?}",*counter);

}