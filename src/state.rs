#[derive(Debug)]
pub enum BattleState<ID> {
	StartWait,
	Setup,
	StartSelecting,
	WaitSelecting,
	StartMoving,
	WaitMoving,
	End(ID),
}

impl<ID> Default for BattleState<ID> {
    fn default() -> Self {
        Self::StartWait
    }
}