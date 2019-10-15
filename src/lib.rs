extern crate bio;
#[macro_use]
extern crate serde_json;

use bio::io::gff;
use multimap::MultiMap;
use regex::Regex;
use serde_json::value::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;

type HashGFF = HashMap<String, bio::io::gff::Record>;

pub enum CompareMode {
    Forward,
    Reverse,
}

pub struct CompareGFF {
    data1: Option<HashGFF>,
    data2: Option<HashGFF>,
    record_issues: bool,
}

impl CompareGFF {
    pub fn new() -> Self {
        Self {
            data1: None,
            data2: None,
            record_issues: false,
        }
    }

    pub fn record_issues(&mut self, do_record: bool) {
        self.record_issues = do_record;
    }

    pub fn new_from_files(filename1: &String, filename2: &String) -> Result<Self, Box<dyn Error>> {
        let mut ret = Self::new();
        ret.data1 = Some(ret.read(Box::new(File::open(&filename1)?))?);
        ret.data2 = Some(ret.read(Box::new(File::open(&filename2)?))?);
        Ok(ret)
    }

    pub fn diff(&self) -> Result<Value, Box<dyn Error>> {
        let mut result = json!( {
            "changes" :[]
        });
        self.compare(CompareMode::Forward, &mut result)?;
        self.compare(CompareMode::Reverse, &mut result)?;
        Ok(result)
    }

    pub fn diff_apollo(&self) -> Result<Value, Box<dyn Error>> {
        self.compare_apollo()
    }

    fn read(&self, file: Box<dyn std::io::Read>) -> Result<HashGFF, Box<dyn Error>> {
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

    fn write(&self, file: Box<dyn std::io::Write>, data: &HashGFF) -> Result<(), Box<dyn Error>> {
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
        mode: CompareMode,
        result: &mut Value,
    ) {
        // Does attrs have that key at all?
        if !attrs.contains_key(key) {
            for value in values {
                let action = match mode {
                    CompareMode::Forward => "remove",
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
                    CompareMode::Forward => "add",
                    _ => "remove",
                };
                let j = json!({ "action" : action , "what" : "attribute" , "id" : id , "key":key , "value" : value2 } );
                result["changes"].as_array_mut().unwrap().push(j);
            }
        }

        match mode {
            CompareMode::Forward => {}
            CompareMode::Reverse => {
                for value in values {
                    if !values2.contains(&value) {
                        let j = json!({"action" : "add", "what" : "attribute" , "id" : id , "key":key , "value" : value });
                        result["changes"].as_array_mut().unwrap().push(j);
                    }
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

    fn compare(&self, mode: CompareMode, result: &mut Value) -> Result<(), Box<dyn Error>> {
        let (data1, data2) = match (&self.data1, &self.data2) {
            (Some(data1), Some(data2)) => (data1, data2),
            _ => return Err(From::from(format!("Both GFF sets need to be initialized"))),
        };
        for (id, r1) in data1 {
            if data2.contains_key(id) {
                match mode {
                    CompareMode::Forward => {}
                    CompareMode::Reverse => continue,
                }
                let r2 = &data2[id];
                self.compare_basics(&r1, r2, id.as_str())
                    .drain(..)
                    .for_each(|change| result["changes"].as_array_mut().unwrap().push(change));

                let r1a = r1.attributes();
                let r2a = r2.attributes();
                for (key, value) in r1a {
                    self.compare_attributes(&id, key, value, r2a, CompareMode::Forward, result);
                }

                for (key, value) in r2a {
                    self.compare_attributes(&id, key, value, r1a, CompareMode::Reverse, result);
                }
            } else {
                match mode {
                    CompareMode::Forward => {
                        let mut o = json! ({"what":"row" , "action": "remove" , "id":id });
                        let s = serde_json::to_string(&r1).unwrap();
                        o["data"] = json!(s);
                        result["changes"].as_array_mut().unwrap().push(o);
                    }
                    CompareMode::Reverse => {
                        // ???
                    }
                }
            }
        }
        Ok(())
    }

    fn get_root_parent_id(
        &self,
        data: &HashGFF,
        id: &String,
        seen: Option<HashSet<String>>,
    ) -> Option<String> {
        let mut seen: HashSet<String> = seen.unwrap_or(HashSet::new());
        if seen.contains(id) {
            return None; // circular ID chain, oh no
        }
        seen.insert(id.to_string());
        match data.get(id) {
            Some(element) => match element.attributes().get("Parent") {
                Some(parent_id) => self.get_root_parent_id(data, parent_id, Some(seen)),
                None => Some(id.to_string()),
            },
            None => None,
        }
    }

    fn infer_original_id_from_apollo(
        &self,
        data1: &HashGFF,
        data2: &HashGFF,
        apollo_element: &bio::io::gff::Record,
        issues: &mut Vec<String>,
    ) -> Option<String> {
        // Try orig_id
        match apollo_element.attributes().get("orig_id") {
            Some(orig_id) => {
                return match data1.get(orig_id) {
                    Some(_) => Some(orig_id.to_string()),
                    None => {
                        issues.push(format!(
                            "Original ID '{}' given in Apollo GFF is not in full dataset!",
                            orig_id
                        ));
                        None
                    }
                }
            }
            None => {}
        }

        // Find Apollo parent
        let apollo_id = apollo_element.attributes().get("ID")?;
        let apollo_parent_id = self.get_root_parent_id(data2, apollo_id, None)?;
        //let apollo_parent_element = data2.get(&apollo_parent_id)?;

        // Find any other Apollo element with that parent and an orig_id
        let some_apollo_parent_id = Some(apollo_parent_id.to_owned());
        let orig_parent_id = data2
            .iter()
            .filter(|(id, _element)| {
                self.get_root_parent_id(data2, id, None) == some_apollo_parent_id
            }) // Same Apollo parent
            .filter_map(|(_id, element)| element.attributes().get("orig_id")) // with orig_id
            .map(|s| s.to_string())
            .filter(|orig_id| data1.contains_key(orig_id)) // with orig_id that exists in original dataset
            .filter_map(|orig_id| self.get_root_parent_id(data1, &orig_id, None)) // get that original root parent
            .nth(0)?;

        // Get all (sub)children on that parent in the original
        let some_orig_parent_id = Some(orig_parent_id);
        let all_children_orig: HashGFF = data1
            .iter()
            .filter(|(_id, data)| data.seqname() == apollo_element.seqname()) // Same chromosome
            .filter(|(id, _data)| self.get_root_parent_id(data1, id, None) == some_orig_parent_id) // Same root parent
            .map(|(id, data)| (id.to_owned(), data.to_owned()))
            .collect();

        // Try original elements with that parent, of the same type
        let same_type: HashGFF = all_children_orig
            .iter()
            .filter(|(_id, data)| data.feature_type() == apollo_element.feature_type())
            .map(|(id, data)| (id.to_owned(), data.to_owned()))
            .collect();

        // Found one element with the same type and (root) parent in the original data, using that one
        if same_type.len() == 1 {
            return Some(
                same_type
                    .iter()
                    .map(|(id, _data)| id.to_owned())
                    .nth(0)
                    .unwrap(),
            );
        }

        // TODO try location?

        None
    }

    /// data1 is "full" GFF, data2 is Apollo GFF
    fn compare_apollo(&self) -> Result<Value, Box<dyn Error>> {
        let (data1, data2) = match (&self.data1, &self.data2) {
            (Some(data1), Some(data2)) => (data1, data2),
            _ => return Err(From::from(format!("Both GFF sets need to be initialized"))),
        };
        let mut issues: Vec<String> = vec![];
        let mut changes: Vec<Value> = vec![];

        let _re = Regex::new(r"-\d+$").unwrap();

        for (_id, apollo_element) in data2 {
            let _attrs = apollo_element.attributes();
            let original_id = match self.infer_original_id_from_apollo(
                data1,
                data2,
                &apollo_element,
                &mut issues,
            ) {
                Some(id) => id,
                None => {
                    issues.push(format!("No original ID found for {:?}", apollo_element));
                    continue;
                }
            };
            let original_parent_id = match data1.get(&original_id) {
                Some(e) => e.attributes().get("Parent"),
                None => None,
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

        Ok(match self.record_issues {
            true => json!({"changes": changes, "issues": issues}),
            false => json!({ "changes": changes }),
        })
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

        CompareGFF::new().compare_attributes(
            &id,
            &key,
            &values,
            &attrs,
            CompareMode::Forward,
            &mut result,
        );

        let expected = json! ({ "changes" : [ { "action" : "add", "what": "attribute", "id" : id , "key":key , "value" : "value2" } ] });
        assert_eq!(result, expected);
    }
}
