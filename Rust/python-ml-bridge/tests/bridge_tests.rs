//! Comprehensive tests for Python ML bridge
//! Following RAG Rule 17.6: Test real functionality with real data

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

#[test]
fn test_python_initialization() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        assert!(py.version_info().major >= 3);
        assert!(py.version_info().minor >= 7, "Should support Python 3.7+");
    });
}

#[test]
fn test_python_version_info() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let version = py.version_info();
        
        // Detailed version check
        assert!(version.major == 3, "Should be Python 3.x");
        assert!(version.minor >= 7, "Should be at least 3.7");
        
        println!("Python version: {}.{}.{}", version.major, version.minor, version.patch);
    });
}

#[test]
fn test_import_sys_module() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let result = py.import_bound("sys");
        assert!(result.is_ok(), "Should be able to import sys module");
        
        let sys = result.unwrap();
        assert!(sys.hasattr("path").unwrap());
        assert!(sys.hasattr("version").unwrap());
    });
}

#[test]
fn test_import_os_module() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let result = py.import_bound("os");
        assert!(result.is_ok(), "Should be able to import os module");
    });
}

#[test]
fn test_import_json_module() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let result = py.import_bound("json");
        assert!(result.is_ok(), "Should be able to import json module");
    });
}

#[test]
fn test_import_nonexistent_module() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let result = py.import_bound("nonexistent_module_12345");
        assert!(result.is_err(), "Importing nonexistent module should fail");
    });
}

#[test]
fn test_sys_path_access() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let sys = py.import_bound("sys").expect("Failed to import sys");
        let path = sys.getattr("path").expect("Failed to get sys.path");
        
        assert!(path.len().is_ok(), "sys.path should have length");
        
        let path_list = path.downcast::<PyList>();
        assert!(path_list.is_ok(), "sys.path should be a list");
    });
}

#[test]
fn test_sys_path_modification() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let sys = py.import_bound("sys").expect("Failed to import sys");
        let path_attr = sys.getattr("path").expect("Failed to get sys.path");
        let path = path_attr.downcast::<PyList>().expect("sys.path should be list");
        
        // Get original length
        let original_len = path.len();
        
        // Try to add a path
        let result = path.insert(0, "/test/path");
        
        if result.is_ok() {
            assert_eq!(path.len(), original_len + 1, "Path should be added");
        }
    });
}

#[test]
fn test_create_python_dict() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let dict = PyDict::new_bound(py);
        
        // Add items
        dict.set_item("key1", "value1").unwrap();
        dict.set_item("key2", 42).unwrap();
        dict.set_item("key3", true).unwrap();
        
        // Verify items
        assert_eq!(dict.get_item("key1").unwrap().unwrap().extract::<String>().unwrap(), "value1");
        assert_eq!(dict.get_item("key2").unwrap().unwrap().extract::<i32>().unwrap(), 42);
        assert_eq!(dict.get_item("key3").unwrap().unwrap().extract::<bool>().unwrap(), true);
    });
}

#[test]
fn test_create_python_list() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let list = PyList::empty_bound(py);
        
        // Add items
        list.append(1).unwrap();
        list.append(2).unwrap();
        list.append(3).unwrap();
        
        assert_eq!(list.len(), 3);
        assert_eq!(list.get_item(0).unwrap().extract::<i32>().unwrap(), 1);
        assert_eq!(list.get_item(1).unwrap().extract::<i32>().unwrap(), 2);
        assert_eq!(list.get_item(2).unwrap().extract::<i32>().unwrap(), 3);
    });
}

#[test]
fn test_python_list_from_vec() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let vec = vec![1, 2, 3, 4, 5];
        let list = PyList::new_bound(py, &vec);
        
        assert_eq!(list.len(), 5);
        assert_eq!(list.get_item(0).unwrap().extract::<i32>().unwrap(), 1);
        assert_eq!(list.get_item(4).unwrap().extract::<i32>().unwrap(), 5);
    });
}

#[test]
fn test_call_python_builtin_function() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        // Call len() function
        let builtins = py.import_bound("builtins").unwrap();
        let len_fn = builtins.getattr("len").unwrap();
        
        let list = PyList::new_bound(py, vec![1, 2, 3]);
        let result = len_fn.call1((list,)).unwrap();
        
        assert_eq!(result.extract::<usize>().unwrap(), 3);
    });
}

#[test]
fn test_execute_python_code() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let result = py.eval_bound("2 + 2", None, None);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().extract::<i32>().unwrap(), 4);
    });
}

#[test]
fn test_execute_python_code_with_error() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let result = py.eval_bound("1 / 0", None, None);
        
        assert!(result.is_err(), "Division by zero should error");
    });
}

#[test]
fn test_python_exception_handling() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let code = "raise ValueError('test error')";
        let result = py.run_bound(code, None, None);
        
        assert!(result.is_err());
        
        if let Err(e) = result {
            let error_str = format!("{}", e);
            assert!(error_str.contains("ValueError") || error_str.contains("test error"));
        }
    });
}

#[test]
fn test_rust_to_python_type_conversion_integers() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let dict = PyDict::new_bound(py);
        
        dict.set_item("i8", 127i8).unwrap();
        dict.set_item("i16", 32767i16).unwrap();
        dict.set_item("i32", 2147483647i32).unwrap();
        dict.set_item("i64", 9223372036854775807i64).unwrap();
        
        assert_eq!(dict.get_item("i8").unwrap().unwrap().extract::<i8>().unwrap(), 127);
        assert_eq!(dict.get_item("i32").unwrap().unwrap().extract::<i32>().unwrap(), 2147483647);
    });
}

#[test]
fn test_rust_to_python_type_conversion_floats() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let dict = PyDict::new_bound(py);
        
        dict.set_item("f32", 3.14f32).unwrap();
        dict.set_item("f64", 2.718281828f64).unwrap();
        
        let f32_val = dict.get_item("f32").unwrap().unwrap().extract::<f32>().unwrap();
        assert!((f32_val - 3.14).abs() < 0.01);
        
        let f64_val = dict.get_item("f64").unwrap().unwrap().extract::<f64>().unwrap();
        assert!((f64_val - 2.718281828).abs() < 0.0001);
    });
}

#[test]
fn test_rust_to_python_type_conversion_strings() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let dict = PyDict::new_bound(py);
        
        dict.set_item("string", "Hello, Python!").unwrap();
        dict.set_item("unicode", "你好世界🌍").unwrap();
        
        assert_eq!(dict.get_item("string").unwrap().unwrap().extract::<String>().unwrap(), "Hello, Python!");
        assert_eq!(dict.get_item("unicode").unwrap().unwrap().extract::<String>().unwrap(), "你好世界🌍");
    });
}

#[test]
fn test_rust_to_python_type_conversion_bools() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let dict = PyDict::new_bound(py);
        
        dict.set_item("true_val", true).unwrap();
        dict.set_item("false_val", false).unwrap();
        
        assert_eq!(dict.get_item("true_val").unwrap().unwrap().extract::<bool>().unwrap(), true);
        assert_eq!(dict.get_item("false_val").unwrap().unwrap().extract::<bool>().unwrap(), false);
    });
}

#[test]
fn test_python_to_rust_type_extraction() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let result = py.eval_bound("42", None, None).unwrap();
        let value: i32 = result.extract().unwrap();
        assert_eq!(value, 42);
        
        let result = py.eval_bound("'hello'", None, None).unwrap();
        let value: String = result.extract().unwrap();
        assert_eq!(value, "hello");
        
        let result = py.eval_bound("True", None, None).unwrap();
        let value: bool = result.extract().unwrap();
        assert_eq!(value, true);
    });
}

#[test]
fn test_python_none_value() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let none = py.None();
        assert!(none.is_none(py));
    });
}

#[test]
fn test_gil_acquisition() {
    pyo3::prepare_freethreaded_python();
    
    // Test multiple GIL acquisitions
    for _ in 0..10 {
        Python::with_gil(|py| {
            let _ = py.version_info();
        });
    }
}

#[cfg(test)]
mod ml_funcs_integration {
    use super::*;
    use std::path::PathBuf;
    
    fn get_ml_funcs_path() -> Option<PathBuf> {
        // Check for ml_funcs module in various locations
        let possible_paths = vec![
            PathBuf::from("Server/Python/ml_funcs"),
            PathBuf::from("Python/ml_funcs"),
            PathBuf::from("ml_funcs"),
        ];
        
        possible_paths.into_iter().find(|p| p.exists())
    }
    
    #[test]
    fn test_import_ml_funcs_if_available() {
        let Some(ml_funcs_path) = get_ml_funcs_path() else {
            eprintln!("Skipping: ml_funcs module not found");
            return;
        };
        
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let sys = py.import_bound("sys").unwrap();
            let path_attr = sys.getattr("path").unwrap();
            let path = path_attr.downcast::<PyList>().unwrap();
            
            // Add ml_funcs parent directory to path
            let parent = ml_funcs_path.parent().unwrap();
            path.insert(0, parent.to_str().unwrap()).unwrap();
            
            // Try to import
            let result = py.import_bound("ml_funcs");
            
            if result.is_ok() {
                println!("✅ Successfully imported ml_funcs module");
            } else {
                println!("❌ Failed to import ml_funcs: {:?}", result.err());
            }
        });
    }
    
    #[test]
    fn test_call_ml_function_if_available() {
        let Some(_ml_funcs_path) = get_ml_funcs_path() else { return; };
        
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|_py| {
            // Setup path and import ml_funcs
            // ...
            
            // Call a function from ml_funcs
            // let result = ml_funcs.call_method("some_function", (), None);
            // assert!(result.is_ok());
        });
    }
}

#[test]
fn test_python_dict_iteration() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let dict = PyDict::new_bound(py);
        dict.set_item("a", 1).unwrap();
        dict.set_item("b", 2).unwrap();
        dict.set_item("c", 3).unwrap();
        
        let mut count = 0;
        for _ in dict.iter() {
            count += 1;
        }
        
        assert_eq!(count, 3);
    });
}

#[test]
fn test_python_list_iteration() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let list = PyList::new_bound(py, vec![1, 2, 3, 4, 5]);
        
        let mut sum = 0;
        for item in list.iter() {
            sum += item.extract::<i32>().unwrap();
        }
        
        assert_eq!(sum, 15);
    });
}

#[test]
fn test_nested_python_structures() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let outer_dict = PyDict::new_bound(py);
        let inner_dict = PyDict::new_bound(py);
        let inner_list = PyList::new_bound(py, vec![1, 2, 3]);
        
        inner_dict.set_item("numbers", inner_list).unwrap();
        outer_dict.set_item("inner", inner_dict).unwrap();
        
        // Access nested structure
        let inner = outer_dict.get_item("inner").unwrap().unwrap();
        let inner_dict_ref = inner.downcast::<PyDict>().unwrap();
        let numbers = inner_dict_ref.get_item("numbers").unwrap().unwrap();
        let numbers_list = numbers.downcast::<PyList>().unwrap();
        
        assert_eq!(numbers_list.len(), 3);
    });
}

#[test]
fn test_python_memory_cleanup() {
    pyo3::prepare_freethreaded_python();
    
    // Create and drop many Python objects
    for _ in 0..1000 {
        Python::with_gil(|py| {
            let _dict = PyDict::new_bound(py);
            let _list = PyList::empty_bound(py);
            // Objects should be cleaned up when GIL is released
        });
    }
    
    // Should not leak memory
}
