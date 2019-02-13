use bio::io::gff;
use multimap::MultiMap;
use serde_json;
use std::fs::File;

pub fn read_gff_into_data(
    filename: String,
    data: &mut std::collections::HashMap<std::string::String, bio::io::gff::Record>,
) {
    let file = File::open(filename).unwrap();
    let mut reader = gff::Reader::new(file, gff::GffType::GFF3);

    for element in reader.records() {
        if !element.is_ok() {
            continue;
        }
        let e = element.unwrap();
        if !e.attributes().contains_key("ID") {
            continue;
        }
        let id = e.attributes()["ID"].clone();
        if data.contains_key(&id) {
            println!("{:?}", id);
            continue;
        }
        data.insert(id, e);
    }
}

fn compare_gff_attributes(
    id: &String,
    key: &String,
    values: &Vec<String>,
    attrs: &MultiMap<String, String>,
    mode: u8,
    result: &mut json::JsonValue,
) {
    if !attrs.contains_key(key) {
        if mode == 0 {
            // key from old file is not in new file, remove
            for value in values {
                result["changes"].push(object! { "action" => "remove_attribute","tag"=>0 , "id" => id.as_str() , "key"=>key.to_string() , "value" => value.to_string() }).unwrap();
            }
        } else {
            // key from new file is not in old file, add
            for value in values {
                result["changes"].push(object! { "action" => "add_attribute","tag"=>0 , "id" => id.as_str() , "key"=>key.to_string() , "value" => value.to_string() }).unwrap();
            }
        }
        return;
    }

    for (key2, values2) in attrs {
        if key != key2 {
            continue;
        }
        for value2 in values2 {
            let mut found: bool = false;
            for value in values {
                if value == value2 {
                    found = true;
                }
            }
            if found {
                continue;
            }
            if mode == 0 {
                // value2 is in the new but not in the old
                result["changes"].push(object! { "action" => "add_attribute","tag"=>1 , "id" => id.as_str() , "key"=>key.to_string() , "value" => value2.to_string() }).unwrap();
            } else {
                // value2 is in the old but not in the new
                result["changes"].push(object! { "action" => "remove_attribute","tag"=>1 , "id" => id.as_str() , "key"=>key.to_string() , "value" => value2.to_string() }).unwrap();
            }
        }

        for value in values {
            let mut found: bool = false;
            for value2 in values2 {
                if value == value2 {
                    found = true;
                }
            }
            if found {
                continue;
            }
            if mode == 0 {
                // value is in the old but not in the new
                // HAD THAT?
                //                result["changes"].push(object! { "action" => "remove_attribute","tag"=>2 , "id" => id.as_str() , "key"=>key.to_string() , "value" => value.to_string() }).unwrap();
            } else {
                // value is in the new but not in the old
                result["changes"].push(object! { "action" => "add_attribute","tag"=>2 , "id" => id.as_str() , "key"=>key.to_string() , "value" => value.to_string() }).unwrap();
            }
        }
    }
}

pub fn compare_gff(
    data1: &std::collections::HashMap<std::string::String, bio::io::gff::Record>,
    data2: &std::collections::HashMap<std::string::String, bio::io::gff::Record>,
    mode: u8,
    result: &mut json::JsonValue,
) {
    for (id, r1) in data1 {
        if data2.contains_key(id) {
            if mode == 1 {
                continue;
            }
            let r2 = data2[id].clone();
            if r1.seqname() != r2.seqname() {
                result["changes"]
                    .push(object! { "action" => "alter" , "id" => id.as_str() , "key" => "seqname" , "value" => r2.seqname() })
                    .unwrap();
            }
            if r1.source() != r2.source() {
                result["changes"]
                    .push(object! { "action" => "alter" , "id" => id.as_str() , "key" => "source" , "value" => r2.source() })
                    .unwrap();
            }

            if r1.feature_type() != r2.feature_type() {
                result["changes"]
                    .push(object! { "action" => "alter" , "id" => id.as_str() , "key" => "feature_type" , "value" => r2.feature_type() })
                    .unwrap();
            }

            if r1.start() != r2.start() {
                result["changes"]
                    .push(object! { "action" => "alter" , "id" => id.as_str() , "key" => "start" , "value" => r2.start().to_string() })
                    .unwrap();
            }

            if r1.end() != r2.end() {
                result["changes"]
                    .push(object! { "action" => "alter" , "id" => id.as_str() , "key" => "end" , "value" => r2.end().to_string() })
                    .unwrap();
            }

            if r1.score() != r2.score() {
                result["changes"]
                    .push(object! { "action" => "alter" , "id" => id.as_str() , "key" => "score" , "value" => r2.score() })
                    .unwrap();
            }

            if r1.strand() != r2.strand() {
                let mut strand: String;
                strand = ".".to_string();
                let s = r2.strand();
                if s.is_some() {
                    strand = s.unwrap().strand_symbol().to_string();
                }
                result["changes"]
                    .push(object! { "action" => "alter" , "id" => id.as_str() , "key" => "strand" , "value" => strand })
                    .unwrap();
            }

            if r1.frame() != r2.frame() {
                result["changes"]
                    .push(object! { "action" => "alter" , "id" => id.as_str() , "key" => "frame" , "value" => r2.frame() })
                    .unwrap();
            }

            let r1a = r1.attributes();
            let r2a = r2.attributes();
            for (key, value) in r1a {
                compare_gff_attributes(id, key, value, r2a, 0, result);
            }

            for (key, value) in r2a {
                compare_gff_attributes(id, key, value, r1a, 1, result);
            }
        } else {
            if mode == 0 {
                result["changes"]
                    .push(object! { "action" => "remove" , "id" => id.as_str()})
                    .unwrap();
            } else {
                let mut data = json::JsonValue::new_object();
                let s = serde_json::to_string(r1).unwrap();
                data["action"] = "add".into();
                data["id"] = id.as_str().into();
                data["data"] = s.into();
            }
        }
    }
}
