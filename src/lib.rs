extern crate bio;
#[macro_use]
extern crate serde_json;

use bio::io::gff;
use multimap::MultiMap;
use serde_json::value::Value;

use std::fs::File;

pub fn read_gff_into_data(
    filename: String,
    data: &mut std::collections::HashMap<std::string::String, bio::io::gff::Record>,
) {
    let file = File::open(&filename).unwrap();
    let mut reader = gff::Reader::new(file, gff::GffType::GFF3);

    for element in reader.records() {
        if !element.is_ok() {
            continue;
        }
        let mut e = element.unwrap();
        if !e.attributes().contains_key("ID") {
            continue;
        }
        let id = e.attributes()["ID"].clone();
        if data.contains_key(&id) {
            println!("Double ID: {:?}", id);
            let _attrs = e.attributes_mut();
            //attrs["ID"] = "xxxx".to_string(); // TODO FIXME
            continue;
        }
        data.insert(id, e);
    }
    if data.is_empty() {
        panic!("Empty file or no gff file: {}", filename)
    }
}

fn compare_gff_attributes(
    id: &String,
    key: &String,
    values: &Vec<String>,
    attrs: &MultiMap<String, String>,
    mode: u8,
    result: &mut Value,
) {
    // Does attrs have that key at all?
    if !attrs.contains_key(key) {
        for value in values {
            let action = match mode {
                0 => "remove",
                _ => "add",
            };
            let j = json!( {"action" : action , "what": "attribute" , "id" : id.as_str() , "key":key.to_string() , "value" : value.to_string() } );
            result["changes"].as_array_mut().unwrap().push(j);
        }
        return;
    }

    // attrs has the key, compare values
    let values2 = attrs.get_vec(key).unwrap();

    for value2 in values2 {
        if !values.contains(&value2) {
            let action = match mode {
                0 => "add",
                _ => "remove",
            };
            let j = json!({ "action" : action , "what" : "attribute" , "id" : id.as_str() , "key":key.to_string() , "value" : value2.to_string() } );
            result["changes"].as_array_mut().unwrap().push(j);
        }
    }

    if mode == 1 {
        for value in values {
            if !values2.contains(&value) {
                let j = json!({"action" : "add", "what" : "attribute" , "id" : id.as_str() , "key":key.to_string() , "value" : value.to_string() });
                result["changes"].as_array_mut().unwrap().push(j);
            }
        }
    }
}

pub fn compare_gff(
    data1: &std::collections::HashMap<std::string::String, bio::io::gff::Record>,
    data2: &std::collections::HashMap<std::string::String, bio::io::gff::Record>,
    mode: u8,
    result: &mut Value,
) {
    for (id, r1) in data1 {
        if data2.contains_key(id) {
            if mode == 1 {
                continue;
            }
            let r2 = data2[id].clone();
            if r1.seqname() != r2.seqname() {
                let j = json!({ "action" : "update" , "what" : "row" , "id" : id.as_str() , "key" : "seqname" , "value" : r2.seqname() });
                result["changes"].as_array_mut().unwrap().push(j);
            }
            if r1.source() != r2.source() {
                let j = json!( { "action" : "update" , "what" : "row" , "id" : id.as_str() , "key" : "source" , "value" : r2.source() });
                result["changes"].as_array_mut().unwrap().push(j);
            }
            if r1.feature_type() != r2.feature_type() {
                let j = json!( { "action" : "update" , "what" : "row" , "id" : id.as_str() , "key" : "feature_type" , "value" : r2.feature_type() });
                result["changes"].as_array_mut().unwrap().push(j);
            }
            if r1.start() != r2.start() {
                let j = json!( { "action" : "update" , "what" : "row" , "id" : id.as_str() , "key" : "start" , "value" : r2.start().to_string() });
                result["changes"].as_array_mut().unwrap().push(j);
            }
            if r1.end() != r2.end() {
                let j = json!( { "action" : "update" , "what" : "row" , "id" : id.as_str() , "key" : "end" , "value" : r2.end().to_string() });
                result["changes"].as_array_mut().unwrap().push(j);
            }
            if r1.score() != r2.score() {
                let j = json!( { "action" : "update" , "what" : "row" , "id" : id.as_str() , "key" : "score" , "value" : r2.score() });
                result["changes"].as_array_mut().unwrap().push(j);
            }
            if r1.strand() != r2.strand() {
                let mut strand: String;
                strand = ".".to_string();
                let s = r2.strand();
                if s.is_some() {
                    strand = s.unwrap().strand_symbol().to_string();
                }
                let j = json!( { "action" : "update" , "what" : "row" , "id" : id.as_str() , "key" : "strand" , "value" : strand });
                result["changes"].as_array_mut().unwrap().push(j);
            }
            if r1.frame() != r2.frame() {
                let j = json!( { "action" : "update" , "what" : "row" , "id" : id.as_str() , "key" : "frame" , "value" : r2.frame() });
                result["changes"].as_array_mut().unwrap().push(j);
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
            let mut o = json! ({"what":"row" , "action": if mode==0 {"remove"} else {"add"} , "id":id.as_str() });
            let s = serde_json::to_string(r1).unwrap();
            o["data"] = json!(s);
            result["changes"].as_array_mut().unwrap().push(o);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attribute_added() {
        let id: String = "the_id".to_string();
        let key: String = "the_key".to_string();
        let values = vec!["value1".to_string(), "value3".to_string()];
        let mut attrs = MultiMap::new();
        let mut result = json! ({"changes":[]});

        attrs.insert("the_key".to_string(), "value1".to_string());
        attrs.insert("the_key".to_string(), "value2".to_string());
        attrs.insert("the_key".to_string(), "value3".to_string());

        compare_gff_attributes(&id, &key, &values, &attrs, 0, &mut result);

        let expected = json! ({ "changes" : [ { "action" : "add", "what": "attribute", "id" : id , "key":key , "value" : "value2" } ] });
        assert_eq!(result, expected);
    }
}
