use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct OrderLine {
    orderid: String,
    sku: String,
    qty: i32,
}

#[derive(Debug)]
struct Batch {
    reference: String,
    sku: String,
    purchased_quantity: i32,
    allocations: HashSet<OrderLine>,
}

impl Batch {
    fn allocate(&mut self, line: &OrderLine) {
        if self.can_allocate(line) {
            self.allocations.insert(line.clone());
        }
    }

    fn can_allocate(&self, line: &OrderLine) -> bool {
        self.sku == line.sku && self.available_quantity() >= line.qty
    }

    fn allocated_quantity(&self) -> i32 {
        self.allocations.iter().map(|line| line.qty).sum()
    }

    fn available_quantity(&self) -> i32 {
        self.purchased_quantity - self.allocated_quantity()
    }
}

fn allocate(line: &OrderLine, batches: &mut Vec<Batch>) -> Result<String, OutOfStock> {
    if let Some(batch) = batches.iter_mut().find(|batch| batch.can_allocate(line)) {
        batch.allocate(line);
        Ok(batch.reference.clone())
    } else {
        Err(OutOfStock)
    }
}

#[derive(Debug)]
struct OutOfStock;

fn main() {
    let lines =[
        OrderLine {
            orderid: String::from("order1"),
            sku: String::from("sku1"),
            qty: 20,
        },
        OrderLine {
            orderid: String::from("order2"),
            sku: String::from("sku2"),
            qty: 5,
        },
        OrderLine {
            orderid: String::from("order1"),
            sku: String::from("sku3"),
            qty: 11,
        },
     ];

    let mut batches = vec![
        Batch {
            reference: String::from("batch1"),
            sku: String::from("sku1"),
            purchased_quantity: 15,
            allocations: HashSet::new(),
        },
        Batch {
            reference: String::from("batch2"),
            sku: String::from("sku1"),
            purchased_quantity: 20,
            allocations: HashSet::new(),
        },
        Batch {
            reference: String::from("batch3"),
            sku: String::from("sku2"),
            purchased_quantity: 5,
            allocations: HashSet::new(),
        },
        Batch {
            reference: String::from("batch4"),
            sku: String::from("sku3"),
            purchased_quantity: 10,
            allocations: HashSet::new(),
        },
    ];


    for line in lines.iter() {
        match allocate(line, &mut batches) {
            Ok(reference) => println!("Allocated batch reference: {}", reference),
            Err(_) => println!("Out of stock for SKU: {}", line.sku),
        }
    }
}