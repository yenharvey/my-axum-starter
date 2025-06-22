use crate::AppState;

pub trait FromState {
    fn from_state(state: &AppState) -> Self;
}
