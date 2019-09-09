extern crate bio;
#[macro_use]
extern crate serde_json;

use bio::io::gff;
use multimap::MultiMap;
use regex::Regex;
use serde_json::value::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

type HashGFF = HashMap<String, bio::io::gff::Record>;

pub struct CompareGFF {
    data1: Option<HashGFF>,
    data2: Option<HashGFF>,
}

impl CompareGFF {
    pub fn new() -> Self {
        Self {
            data1: None,
            data2: None,
        }
    }

    pub fn new_from_files(filename1: &String, filename2: &String) -> Result<Self, Box<dyn Error>> {
        let mut ret = Self::new();
        ret.data1 = Some(ret.read(Box::new(File::open(&filename1)?))?);
        ret.data2 = Some(ret.read(Box::new(File::open(&filename2)?))?);
        Ok(ret)
    }

    /*
    pub fn data1(&self) -> &Option<HashGFF> {
        &self.data1
    }

    pub fn data2(&self) -> &Option<HashGFF> {
        &self.data2
    }
    */

    pub fn diff(&self) -> Result<Value, Box<dyn Error>> {
        let mut result = json!( {
            "changes" :[]
        });
        self.compare(0, &mut result)?;
        self.compare(1, &mut result)?;
        Ok(result)
    }

    pub fn diff_apollo(&self) -> Result<Value, Box<dyn Error>> {
        let diff = self.compare_apollo()?;
        //cg.apply_diff(&mut full, &diff); // TODO FIXME

        //let file = File::create(filename)?;
        // io::stdout()
        //cg.write(Box::new(io::stdout()), &full)?; // TODO FIXME

        Ok(diff)
    }

    pub fn read(&self, file: Box<dyn std::io::Read>) -> Result<HashGFF, Box<dyn Error>> {
        let mut ret: HashMap<String, bio::io::gff::Record> = HashMap::new();
        let mut reader = gff::Reader::new(file, gff::GffType::GFF3);

        for element in reader.records() {
            if !element.is_ok() {
                continue;
            }
            let mut e = match element {
                Ok(e) => e,
                _ => continue,
            };
            if !e.attributes().contains_key("ID") {
                continue;
            }
            let id = e.attributes()["ID"].clone();
            if ret.contains_key(&id) {
                println!("Double ID: {:?}", id);
                let _attrs = e.attributes_mut();
                //attrs["ID"] = "xxxx".to_string(); // TODO FIXME
                continue;
            }
            ret.insert(id, e);
        }
        if ret.is_empty() {
            return Err(From::from(format!("Empty file or no gff file")));
        }
        Ok(ret)
    }

    pub fn write(
        &self,
        file: Box<dyn std::io::Write>,
        data: &HashGFF,
    ) -> Result<(), Box<dyn Error>> {
        let mut writer = gff::Writer::new(file, gff::GffType::GFF3);
        for (_k, v) in data {
            writer.write(v)?;
        }
        Ok(())
    }

    pub fn write_data1(&self, file: Box<dyn std::io::Write>) -> Result<(), Box<dyn Error>> {
        match &self.data1 {
            Some(data1) => self.write(file, &data1),
            None => Err(From::from(format!("write_data1:: data1 is not set"))),
        }
    }

    fn compare_attributes(
        &self,
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
                let j = json!( {"action" : action , "what": "attribute" , "id" : id , "key":key.to_string() , "value" : value.to_string() } );
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
                let j = json!({ "action" : action , "what" : "attribute" , "id" : id , "key":key , "value" : value2 } );
                result["changes"].as_array_mut().unwrap().push(j);
            }
        }

        if mode == 1 {
            for value in values {
                if !values2.contains(&value) {
                    let j = json!({"action" : "add", "what" : "attribute" , "id" : id , "key":key , "value" : value });
                    result["changes"].as_array_mut().unwrap().push(j);
                }
            }
        }
    }

    fn compare_basics(
        &self,
        r1: &bio::io::gff::Record,
        r2: &bio::io::gff::Record,
        id: &str,
    ) -> Vec<Value> {
        let mut changes: Vec<Value> = vec![];
        if r1.seqname() != r2.seqname() {
            let j = json!({ "action" : "update" , "what" : "row" , "id" : id , "key" : "seqname" , "value" : r2.seqname() });
            changes.push(j);
        }
        if r1.source() != r2.source() {
            let j = json!( { "action" : "update" , "what" : "row" , "id" : id , "key" : "source" , "value" : r2.source() });
            changes.push(j);
        }
        if r1.feature_type() != r2.feature_type() {
            let j = json!( { "action" : "update" , "what" : "row" , "id" : id , "key" : "feature_type" , "value" : r2.feature_type() });
            changes.push(j);
        }
        if r1.start() != r2.start() {
            let j = json!( { "action" : "update" , "what" : "row" , "id" : id , "key" : "start" , "value" : r2.start().to_string() });
            changes.push(j);
        }
        if r1.end() != r2.end() {
            let j = json!( { "action" : "update" , "what" : "row" , "id" : id , "key" : "end" , "value" : r2.end().to_string() });
            changes.push(j);
        }
        if r1.score() != r2.score() {
            let j = json!( { "action" : "update" , "what" : "row" , "id" : id , "key" : "score" , "value" : r2.score() });
            changes.push(j);
        }
        if r1.strand() != r2.strand() {
            let mut strand: String;
            strand = ".".to_string();
            let s = r2.strand();
            if s.is_some() {
                strand = s.unwrap().strand_symbol().to_string();
            }
            let j = json!( { "action" : "update" , "what" : "row" , "id" : id , "key" : "strand" , "value" : strand });
            changes.push(j);
        }
        if r1.frame() != r2.frame() {
            let j = json!( { "action" : "update" , "what" : "row" , "id" : id , "key" : "frame" , "value" : r2.frame() });
            changes.push(j);
        }
        changes
    }

    fn compare(&self, mode: u8, result: &mut Value) -> Result<(), Box<dyn Error>> {
        let (data1, data2) = match (&self.data1, &self.data2) {
            (Some(data1), Some(data2)) => (data1, data2),
            _ => return Err(From::from(format!("Both GFF sets need to be initialized"))),
        };
        for (id, r1) in data1 {
            if data2.contains_key(id) {
                if mode == 1 {
                    continue;
                }
                let r2 = &data2[id];
                self.compare_basics(&r1, r2, id.as_str())
                    .drain(..)
                    .for_each(|change| result["changes"].as_array_mut().unwrap().push(change));

                let r1a = r1.attributes();
                let r2a = r2.attributes();
                for (key, value) in r1a {
                    self.compare_attributes(&id, key, value, r2a, 0, result);
                }

                for (key, value) in r2a {
                    self.compare_attributes(&id, key, value, r1a, 1, result);
                }
            } else {
                let mut o = json! ({"what":"row" , "action": if mode==0 {"remove"} else {"add"} , "id":id });
                let s = serde_json::to_string(&r1).unwrap();
                o["data"] = json!(s);
                result["changes"].as_array_mut().unwrap().push(o);
            }
        }
        Ok(())
    }

    /// data1 is "full" GFF, data2 is Apollo GFF
    fn compare_apollo(&self) -> Result<Value, Box<dyn Error>> {
        let (data1, data2) = match (&self.data1, &self.data2) {
            (Some(data1), Some(data2)) => (data1, data2),
            _ => return Err(From::from(format!("Both GFF sets need to be initialized"))),
        };
        let mut issues: Vec<String> = vec![];
        let mut changes: Vec<Value> = vec![];

        let re = Regex::new(r"-\d+$").unwrap();

        for (id, apollo_element) in data2 {
            let attrs = apollo_element.attributes();
            let mut original_id: Option<String> = None;
            let mut original_parent_id: Option<String> = None;
            if attrs.contains_key("orig_id") {
                let orig_id = attrs["orig_id"].clone();
                if !data1.contains_key(&orig_id) {
                    issues.push(format!("Original ID {} not in full dataset!", &orig_id));
                    continue;
                }
                original_id = Some(orig_id);
            } else if attrs.contains_key("Parent") {
                let parent = attrs["Parent"].clone();
                if !data2.contains_key(&parent) {
                    issues.push(format!(
                        "Parent {} of {} not in Apollo dataset!",
                        &parent, &id
                    ));
                    continue;
                }
                let parent_row = data2.get(&parent).unwrap();
                if !parent_row.attributes().contains_key("Name") {
                    issues.push(format!(
                        "Parent {} of {} has no 'Name' attribute",
                        &parent, &id
                    ));
                    continue;
                }
                let parent_id = parent_row.attributes().get("Name").unwrap();
                let parent_id = re.replace(parent_id, "");
                original_parent_id = Some(parent_id.to_string());
            } else {
                let row_id = attrs.get("Name").unwrap();
                let row_id = re.replace(row_id, "");
                issues.push(format!("Top-level row {} is {}", &id, &row_id));
                // row_id => original_id?
            }
            //println!("{:?}/{:?}", &original_id, &original_parent_id);
            let original_id = match original_id {
                Some(id) => id,
                None => {
                    issues.push(format!("No original ID found for {:?}", apollo_element));
                    continue;
                }
            };
            let original_element = match data1.get(&original_id) {
                Some(e) => e,
                None => {
                    issues.push(format!("No original element found for {}", &original_id));
                    continue;
                }
            };
            //println!("!!{:?}", &original_element);

            // Add/remove/change parent ID
            match (
                original_parent_id,
                original_element.attributes().get("Parent"),
            ) {
                (Some(apollo_opid), Some(original_opid)) => {
                    if *apollo_opid != *original_opid {
                        let j = json!({ "action":"update" , "what":"attribute" , "id" : original_id , "key":"Parent" , "value" : apollo_opid } );
                        changes.push(j);
                        let j = json!({ "action":"remove" , "what":"attribute" , "id" : original_id , "key":"Parent" , "value" : original_opid } );
                        changes.push(j);
                    }
                }
                (Some(apollo_opid), None) => {
                    let j = json!({ "action":"add" , "what":"attribute" , "id" : original_id , "key":"Parent" , "value" : apollo_opid } );
                    changes.push(j);
                }
                (None, Some(original_opid)) => {
                    let _j = json!({ "action":"remove" , "what":"attribute" , "id" : original_id , "key":"Parent" , "value" : original_opid } );
                    //changes.push(j); // IGNORE THIS
                }
                (None, None) => {}
            }

            self.compare_basics(&original_element, &apollo_element, original_id.as_str())
                .drain(..)
                .filter(|change| {
                    match (
                        change["action"].as_str(),
                        change["key"].as_str(),
                        change["value"].as_str(),
                    ) {
                        (Some("update"), Some("source"), Some(".")) => false,
                        _ => true, // Default
                    }
                })
                .for_each(|change| changes.push(change));
        }

        let diff = json!({"changes": changes, "issues": issues});
        Ok(diff)
    }

    pub fn apply_diff(&mut self, diff: &Value) -> Result<&HashGFF, Box<dyn Error>> {
        let changes = match diff["changes"].as_array() {
            Some(changes) => changes,
            None => return Err(From::from(format!("No changes in diff"))),
        };
        let data = match self.data1.as_mut() {
            Some(data) => data,
            _ => return Err(From::from(format!("GFF set 1 needs to be initialized"))),
        };
        changes
            .iter()
            .for_each(|change| match change["action"].as_str() {
                Some("remove") => {
                    // TODO
                }
                Some("update") | Some("add") => {
                    match change["what"].as_str() {
                        Some("row") => match change["id"].as_str() {
                            Some(id) => {
                                let element = match data.get_mut(id) {
                                    Some(element) => element,
                                    None => {
                                        eprintln!(
                                            "apply_diff: ID {} does not appear in data set",
                                            &id
                                        );
                                        return;
                                    }
                                };
                                let value = match change["value"].as_str() {
                                    Some(v) => v,
                                    None => {
                                        eprintln!("apply_diff: No value in {:?}", &change);
                                        return;
                                    }
                                };
                                match change["key"].as_str() {
                                    Some("seqname") => *element.seqname_mut() = value.to_string(),
                                    Some("source") => *element.source_mut() = value.to_string(),
                                    Some("feature_type") => {
                                        *element.feature_type_mut() = value.to_string()
                                    }
                                    Some("start") => {
                                        *element.start_mut() = value.parse::<u64>().unwrap()
                                    }
                                    Some("end") => {
                                        *element.end_mut() = value.parse::<u64>().unwrap()
                                    }
                                    Some("score") => *element.score_mut() = value.to_string(),
                                    Some("strand") => *element.strand_mut() = value.to_string(),
                                    Some("frame") => *element.frame_mut() = value.to_string(),
                                    _ => eprintln!(
                                        "apply_diff: Unknown/missing 'key' in {:?}",
                                        change
                                    ),
                                }
                            }
                            None => eprintln!("apply_diff: Missing 'id' in {:?}", change),
                        },
                        Some("attribute") => {
                            // Todo
                            // Differenciate between add and update?
                        }
                        _ => {
                            eprintln!("apply_diff: Unknown/missing 'what' in {:?}", change);
                        }
                    }
                }
                Some(other) => {
                    eprintln!("apply_diff: Unknown action {} in {:?}", other, change);
                }
                _ => {
                    eprintln!("apply_diff: No action in {:?}", change);
                }
            });
        Ok(data)
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

        CompareGFF::new().compare_attributes(&id, &key, &values, &attrs, 0, &mut result);

        let expected = json! ({ "changes" : [ { "action" : "add", "what": "attribute", "id" : id , "key":key , "value" : "value2" } ] });
        assert_eq!(result, expected);
    }
}
