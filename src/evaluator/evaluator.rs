pub type Error = Box<dyn std::error::Error>;

pub struct BooleanEvaluationResult {
    error: Option<Error>,
    result: bool
}

impl BooleanEvaluationResult {
    pub fn create_completed(result: bool) -> Self {
        Self {
            error: None,
            result
        }
    }
    pub fn create_faulty(error: Error) -> Self {
        Self {
            error: Some(error),
            result: false
        }
    }
    #[inline]
    pub fn is_faulty(&self) -> bool {
        self.error.is_some()
    }
    pub fn get_result(self) -> Result<bool, Error> {
        if self.is_faulty(){
            Err(self.error.unwrap())
        } else {
            Ok(self.result)
        }
    }
}

pub trait IRustapoWrappedInstance {
    fn evaluate_boolean(&self, expr: &String) -> BooleanEvaluationResult;
    fn declare_f64(&self, name: &String, value: f64) -> Result<(), Error>;
    fn declare_string(&self, name: &String, value: String) -> Result<(), Error>;
}


pub trait IRustapoEvaluator<TInstance> where TInstance: IRustapoWrappedInstance {
    fn capture(&self) -> TInstance;
}