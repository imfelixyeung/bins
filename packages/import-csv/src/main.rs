use bytes::{Buf, Bytes};
use futures_util::StreamExt;

#[tokio::main]
async fn main() {
    // connect to database
    // let mut client =
    //     postgres::Client::connect("host=localhost user=postgres", postgres::NoTls).unwrap();

    // request the csv file
    let request_url = format!(
        "https://opendata.leeds.gov.uk/downloads/bins/{file}.csv",
        file = "dm_premises"
    );
    println!("Fetching {}", request_url);

    let client = reqwest::Client::new();
    let response = client
        .get(request_url)
        .send()
        .await
        .expect("Unable to fetch csv file");

    println!("Fetch finished");

    // get the byte stream
    let mut stream = response.bytes_stream();

    let mut counter = 0;

    // hold content that were not processed from previous loop
    let mut previous_content: Vec<u8> = vec![];

    while let Some(item) = stream.next().await {
        // get the current chunk and convert into vector of u8
        let item_content = item.expect("unknown item").to_vec();
        let mut content: Vec<_> = item_content
            .iter()
            .filter(|x| x != &&0x0)
            .cloned()
            .collect();

        // insert the previous contents into start of this chunk for further processing
        if previous_content.len() > 0 {
            content.splice(0..0, previous_content);
        }

        println!("Chunk {} size: {}", counter, content.len());
        counter += 1;

        let lines: Vec<_> = content
            .split(|b| b == &0xA)
            .map(|line| line.strip_suffix(&[0xD]).unwrap_or(line))
            .map(|slice| slice.to_vec())
            .collect();

        let last_line = lines.last().expect("last line not found");
        // previous_content = &last_line.to_vec();
        previous_content = last_line.clone();

        let current_content: Vec<_> = Vec::from(&lines[..lines.len() - 1])
            .iter()
            .flat_map(|slice| {
                let mut new = slice.to_vec();
                new.push(0xA);
                return new;
            })
            .collect();

        // println!(
        //     "Current: {}",
        //     String::from_utf8(current_content.to_vec()).unwrap()
        // );

        let mut reader = csv::Reader::from_reader(current_content.reader());

        for result in reader.records() {
            let record = result.expect("error reading record");

            // println!("{:?}", record);

            let premises_id = record.get(0).unwrap();
            let address_room = record.get(1).unwrap();
            let address_number = record.get(2).unwrap();
            let address_street = record.get(3).unwrap();
            let address_locality = record.get(4).unwrap();
            let address_city = record.get(5).unwrap();
            let address_postcode = record.get(6).unwrap();

            // println!(
            //     "{},{},{},{},{},{},{}",
            //     premises_id,
            //     address_room,
            //     address_number,
            //     address_street,
            //     address_locality,
            //     address_city,
            //     address_postcode
            // );
        }

        // if counter == 10 {
        //     break;
        // };
    }

    println!("Done!");
}
