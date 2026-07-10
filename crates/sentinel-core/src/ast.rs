use std::any::Any;
use std::fmt::Debug;
use std::path::PathBuf;

pub trait Ast: Send + Sync + Debug {
    fn files(&self) -> &[PathBuf];

    fn as_any(&self) -> &dyn Any;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    use std::path::PathBuf;

    #[derive(Debug)]
    struct TestProject {
        paths: Vec<PathBuf>,
    }

    impl Ast for TestProject {
        fn files(&self) -> &[PathBuf] {
            &self.paths
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn ast_files() {
        let project = TestProject {
            paths: vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")],
        };
        assert_eq!(project.files().len(), 2);
    }

    #[test]
    fn ast_downcast() {
        let project = TestProject { paths: vec![] };
        let ast: &dyn Ast = &project;
        let downcasted = ast.as_any().downcast_ref::<TestProject>();
        assert!(downcasted.is_some());
    }

    #[test]
    fn ast_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Box<dyn Ast>>();
        assert_sync::<Box<dyn Ast>>();
    }
}
