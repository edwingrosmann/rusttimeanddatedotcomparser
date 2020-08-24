use std::collections::HashMap;

use async_std::stream::StreamExt;
use bson;
use bson::from_bson;
use chrono::Utc;
use mongodb::bson::{doc, Bson, Document};
use mongodb::Client;

use crate::parse_timeanddate_dot_com::TimeData;

pub fn load_stored_time_data() -> HashMap<String, TimeData> {
    //Create a MongoDB-client...
    let client = create_mongo_client("localhost").expect("Could not create aMongoDB Client");

    let start = Utc::now();

    let all_time_data = read_all(&client, "test", "city_data")
        .expect("Error Occurred whilst reading all time-data");

    println!("Loading-time for all City-data: {}", Utc::now() - start);

    all_time_data
}

///When storing freshly downloaded time-data, the existing data will be dropped; this will ensure that
/// old pages that are now longer downloaded/updated - because of a change in pages-to-download - do not
/// linger and thusly are inaccurate. Also it prevents no-longer-wanted-pages to show up in the UI.
pub fn replace_stored_data_with(new_data: &HashMap<String, TimeData>) {
    //Create a MongoDB-client...
    let client = create_mongo_client("localhost").expect("Could not create aMongoDB Client");

    //Get rid of the existing data...
    delete_all(&client, "test", "city_data").expect("Could not drop the time-data");

    let start = Utc::now();

    //Store all the cities...
    for page_data in new_data {
        // let s = serde_json::to_value(city).unwrap();
        // let city_doc = &mut serde_json::from_value(s).expect("Creating City-data-document failed");
        upsert(
            &client,
            "test",
            "city_data",
            page_data.0,
            &mut bson::to_bson(&page_data.1)
                .expect("Could not create BSON from city-data")
                .as_document()
                .expect("Could not create Document for City-data")
                .clone(),
        )
        .expect(&format!(
            "Erroed Occurred whilst storing city-data {:?}",
            page_data
        ));
    }
    println!("Storing-time for all City-data: {}", Utc::now() - start);
}

#[tokio::main]
async fn create_mongo_client(host: &str) -> mongodb::error::Result<Client> {
    Ok(Client::with_uri_str(format!("mongodb://{}:27017", host).as_str()).await?)
}

#[tokio::main]
async fn delete_all(
    client: &Client,
    database: &str,
    collection_name: &str,
) -> mongodb::error::Result<()> {
    let collection = client.database(database).collection(collection_name);
    collection.delete_many(Document::default(), None).await?;
    Ok(())
}

#[tokio::main]
async fn upsert(
    client: &Client,
    database: &str,
    collection_name: &str,
    key: &String,
    data: &mut Document,
) -> mongodb::error::Result<()> {
    //Open the collection ...
    let collection = client.database(database).collection(collection_name);

    //Add _id-, city_count- and updated_at-fields...
    add_extra_fields(key, data);

    //Find by key-field...
    let q = &doc! {"_id":data.get_str("_id").unwrap()};

    //Upsert: If the document exists then update otherwise insert.
    //Empirical Observation: Updating and inserting takes about the same...
    if collection.count_documents(q.clone(), None).await? > 0 {
        collection.update_one(q.clone(), data.clone(), None).await?;
        println!(
            "Updated New City-time with key {:?} into collection: {}/{}",
            q, database, collection_name
        );
    } else {
        println!(
            "Inserted New City-time with key {} into collection: {}/{}",
            collection.insert_one(data.clone(), None).await?.inserted_id,
            database,
            collection_name
        );
    }

    Ok(())
}

fn add_extra_fields(key: &String, data: &mut Document) {
    data.insert("_id", key);
    data.insert(
        "count",
        data.get_array("city_times")
            .expect("could not determine the number of city-times")
            .len() as u32,
    );
    data.insert("last_updated", Utc::now().to_rfc3339());
}

#[tokio::main]
async fn read_all(
    client: &Client,
    database: &str,
    collection_name: &str,
) -> mongodb::error::Result<HashMap<String, TimeData>> {
    //Open the collection ...
    let collection = client.database(database).collection(collection_name);

    let mut all_time_data = collection.find(None, None).await?;

    let mut result_map = HashMap::<String, TimeData>::new();

    while let Some(doc) = all_time_data.next().await {
        let d = doc.unwrap();
        let key = d.get_str("_id").unwrap().to_string();
        let bs = Bson::from(&d);
        let td =
            from_bson::<TimeData>(bs).expect("Mongodb Document could not be converted to TimeData");
        println!("{}: {}\n\n\n\n", key, td);

        result_map.insert(key, td);
    }

    Ok(result_map)
}
