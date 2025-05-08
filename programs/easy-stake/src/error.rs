use anchor_lang::prelude::*;


#[error_code]
pub enum StakingError {
    #[msg("错误的储备账户所有者，必须是系统账户")]
    WrongReserveOwner, // 6000 0x1770

    #[msg("储备账户必须没有数据，但发现有数据")]
    NonEmptyReserveData, // 6001 0x1771

    #[msg("无效的初始储备Lamports")]
    InvalidInitialReserveLamports, // 6002 0x1772

    #[msg("验证者块大小为零")]
    ZeroValidatorChunkSize, // 6003 0x1773

    #[msg("验证者块大小过大")]
    TooBigValidatorChunkSize, // 6004 0x1774

    #[msg("信用块大小为零")]
    ZeroCreditChunkSize, // 6005 0x1775

    #[msg("信用块大小过大")]
    TooBigCreditChunkSize, // 6006 0x1776

    #[msg("信用费用过低")]
    TooLowCreditFee, // 6007 0x1777

    #[msg("无效的铸币权限")]
    InvalidMintAuthority, // 6008 0x1778

    #[msg("铸币账户有非空初始供应")]
    MintHasInitialSupply, // 6009 0x1779

    #[msg("无效的所有者费用状态")]
    InvalidOwnerFeeState, // 6010 0x177a

    #[msg(
        "无效的程序ID。若要使用来自其他账户的程序，请更新代码中的ID"
    )]
    InvalidProgramId, // 6011 0x177b

    #[msg("意外的账户")]
    UnexpectedAccount, // 6012 0x177c

    #[msg("计算失败")]
    CalculationFailure, // 6013 0x177d

    #[msg("不能存入带有锁定的质押账户")]
    StakeAccountWithLockup, // 6014 0x177e

    #[msg("最小质押数额过低")]
    MinStakeIsTooLow, // 6015 0x177f

    #[msg("流动性提供者最大费用过高")]
    LpMaxFeeIsTooHigh, // 6016 0x1780

    #[msg("基点溢出")]
    BasisPointsOverflow, // 6017 0x1781

    #[msg("流动性提供者最小费用大于最大费用")]
    LpFeesAreWrongWayRound, // 6018 0x1782

    #[msg("流动性目标过低")]
    LiquidityTargetTooLow, // 6019 0x1783

    #[msg("票据未到期，请等待更多周期")]
    TicketNotDue, // 6020 0x1784

    #[msg("票据未准备好，请等待几小时后重试")]
    TicketNotReady, // 6021 0x1785

    #[msg("错误的票据受益人")]
    WrongBeneficiary, // 6022 0x1786

    #[msg("质押账户尚未更新")]
    StakeAccountNotUpdatedYet, // 6023 0x1787

    #[msg("质押账户未委托")]
    StakeNotDelegated, // 6024 0x1788

    #[msg("质押账户处于紧急解除质押状态")]
    StakeAccountIsEmergencyUnstaking, // 6025 0x1789

    #[msg("流动性池中流动性不足")]
    InsufficientLiquidity, // 6026 0x178a

    // Not used anymore
    // #[msg("Auto adding a validator is not enabled")]
    // AutoAddValidatorIsNotEnabled, // 6027 0x178b
    NotUsed6027, // 6027 0x178b

    #[msg("无效的管理员权限")]
    InvalidAdminAuthority, // 6028 0x178c

    #[msg("无效的验证者系统管理员")]
    InvalidValidatorManager, // 6029 0x178d

    #[msg("无效的质押列表账户标识符")]
    InvalidStakeListDiscriminator, // 6030 0x178e

    #[msg("无效的验证者列表账户标识符")]
    InvalidValidatorListDiscriminator, // 6031 0x178f

    #[msg("国库分成过高")]
    TreasuryCutIsTooHigh, // 6032 0x1790

    #[msg("奖励费用过高")]
    RewardsFeeIsTooHigh, // 6033 0x1791

    #[msg("质押已达到上限")]
    StakingIsCapped, // 6034 0x1792

    #[msg("流动性已达到上限")]
    LiquidityIsCapped, // 6035 0x1793

    #[msg("更新窗口过低")]
    UpdateWindowIsTooLow, // 6036 0x1794

    #[msg("最小提现额过高")]
    MinWithdrawIsTooHigh, // 6037 0x1795

    #[msg("提现金额过低")]
    WithdrawAmountIsTooLow, // 6038 0x1796

    #[msg("存款金额过低")]
    DepositAmountIsTooLow, // 6039 0x1797

    #[msg("用户资金不足")]
    NotEnoughUserFunds, // 6040 0x1798

    #[msg("错误的代币所有者或代理")]
    WrongTokenOwnerOrDelegate, // 6041 0x1799

    #[msg("质押增减过早")]
    TooEarlyForStakeDelta, // 6042 0x179a

    #[msg("需要委托的质押")]
    RequiredDelegatedStake, // 6043 0x179b

    #[msg("需要激活的质押")]
    RequiredActiveStake, // 6044 0x179c

    #[msg("需要正在解除质押的质押")]
    RequiredDeactivatingStake, // 6045 0x179d

    #[msg("存入未激活的质押")]
    DepositingNotActivatedStake, // 6046 0x179e

    #[msg("存入的质押委托过低")]
    TooLowDelegationInDepositingStake, // 6047 0x179f

    #[msg("错误的存入质押余额")]
    WrongStakeBalance, // 6048 0x17a0

    #[msg("错误的验证者账户或索引")]
    WrongValidatorAccountOrIndex, // 6049 0x17a1

    #[msg("错误的质押账户或索引")]
    WrongStakeAccountOrIndex, // 6050 0x17a2

    #[msg("质押增减为正，应执行质押操作而非解除质押")]
    UnstakingOnPositiveDelta, // 6051 0x17a3

    #[msg("质押增减为负，应执行解除质押操作而非质押")]
    StakingOnNegativeDelta, // 6052 0x17a4

    #[msg("在周期内移动质押受到限制")]
    MovingStakeIsCapped, // 6053 0x17a5

    #[msg("质押必须是未初始化状态")]
    StakeMustBeUninitialized, // 6054 0x17a6

    // merge stakes
    #[msg("目标质押必须已委托")]
    DestinationStakeMustBeDelegated, // 6055 0x17a7

    #[msg("目标质押不得处于解除状态")]
    DestinationStakeMustNotBeDeactivating, // 6056 0x17a8

    #[msg("目标质押必须已更新")]
    DestinationStakeMustBeUpdated, // 6057 0x17a9

    #[msg("目标质押委托无效")]
    InvalidDestinationStakeDelegation, // 6058 0x17aa

    #[msg("源质押必须已委托")]
    SourceStakeMustBeDelegated, // 6059 0x17ab

    #[msg("源质押不得处于解除状态")]
    SourceStakeMustNotBeDeactivating, // 6060 0x17ac

    #[msg("源质押必须已更新")]
    SourceStakeMustBeUpdated, // 6061 0x17ad

    #[msg("源质押委托无效")]
    InvalidSourceStakeDelegation, // 6062 0x17ae

    #[msg("无效的延迟解除质押票据")]
    InvalidDelayedUnstakeTicket, // 6063 0x17af

    #[msg("重复使用延迟解除质押票据")]
    ReusingDelayedUnstakeTicket, // 6064 0x17b0

    #[msg("从非零评分验证者紧急解除质押")]
    EmergencyUnstakingFromNonZeroScoredValidator, // 6065 0x17b1

    #[msg("错误的验证者重复标志")]
    WrongValidatorDuplicationFlag, // 6066 0x17b2

    #[msg("重新存入marinade质押")]
    RedepositingMarinadeStake, // 6067 0x17b3

    #[msg("移除有余额的验证者")]
    RemovingValidatorWithBalance, // 6068 0x17b4

    #[msg("重新委托将使验证者超过质押目标")]
    RedelegateOverTarget, // 6069 0x17b5

    #[msg("源和目标验证者相同")]
    SourceAndDestValidatorsAreTheSame, // 6070 0x17b6

    #[msg("部分mSOL代币在marinade合约外被铸造")]
    UnregisteredMsolMinted, // 6071 0x17b7

    #[msg("部分LP代币在质押合约外被铸造")]
    UnregisteredLPMinted, // 6072 0x17b8

    #[msg("列表索引越界")]
    ListIndexOutOfBounds, // 6073 0x17b9

    #[msg("列表溢出")]
    ListOverflow, // 6074 0x17ba

    #[msg("请求暂停，但已处于暂停状态")]
    AlreadyPaused, // 6075 0x17bb

    #[msg("请求恢复，但未处于暂停状态")]
    NotPaused, // 6076 0x17bc

    #[msg("紧急暂停已激活")]
    ProgramIsPaused, // 6077 0x17bd

    #[msg("无效的暂停权限")]
    InvalidPauseAuthority, // 6078 0x17be

    #[msg("选定的质押账户资金不足")]
    SelectedStakeAccountHasNotEnoughFunds, // 6079 0x17bf

    #[msg("基点CENTS溢出")]
    BasisPointCentsOverflow, // 6080 0x17c0

    #[msg("提现质押账户未启用")]
    WithdrawStakeAccountIsNotEnabled, // 6081 0x17c1

    #[msg("提现质押账户费用过高")]
    WithdrawStakeAccountFeeIsTooHigh, // 6082 0x17c2

    #[msg("延迟解除质押费用过高")]
    DelayedUnstakeFeeIsTooHigh, // 6083 0x17c3

    #[msg("提现质押账户金额过低")]
    WithdrawStakeLamportsIsTooLow, // 6084 0x17c4

    /// 当提现后余额小于最小质押时
    #[msg("质押账户余额剩余过低")]
    StakeAccountRemainderTooLow, // 6085 0x17c5

    #[msg("列表容量不得小于当前大小")]
    ShrinkingListWithDeletingContents, // 6086 0x17c6

    #[msg("计算溢出")]
    MathOverflow, // 6087 0x17c7

    #[msg("检测到未经授权或存在漏洞的 LP 代币铸造行为")]
    UnauthorizedOrExploitedLPMinting, // 6088 0x17c8
}