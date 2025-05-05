import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EasyStake } from "../target/types/easy_stake";
import { PublicKey } from "@solana/web3.js";
import { AccountLayout, MintLayout } from "@solana/spl-token";

describe("easy-stake", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.easyStake as Program<EasyStake>;

  const payer = provider.wallet.publicKey;


  // 所以公钥
  let stakePoolConfigPda: PublicKey;
  let stakePoolConfigBump: number;
  let msolPda: PublicKey;
  let msolBump: number;
  let lpMintPda: PublicKey;
  let lpMintBump: number;
  let solLegtPda: PublicKey;
  let solLegBump: number;
  let msolLegtPda: PublicKey;
  let msolLegBump: number;

  // additional PDAs
  let operationalSolAccount: PublicKey;

  before(async () => {
    [stakePoolConfigPda, stakePoolConfigBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake_pool")],
      program.programId
    );

    [msolPda, msolBump] = PublicKey.findProgramAddressSync(
      [stakePoolConfigPda.toBuffer(), Buffer.from("msol_mint")],
      program.programId
    );

    [lpMintPda, lpMintBump] = PublicKey.findProgramAddressSync(
      [stakePoolConfigPda.toBuffer(), Buffer.from("lp_mint")],
      program.programId
    );

    [solLegtPda, solLegBump] = PublicKey.findProgramAddressSync(
      [stakePoolConfigPda.toBuffer(), Buffer.from("liq_sol")],
      program.programId
    );

    [msolLegtPda, msolLegBump] = PublicKey.findProgramAddressSync(
      [stakePoolConfigPda.toBuffer(), Buffer.from("liq_st_sol")],
      program.programId
    );


    // operational_sol_account — 这里直接用 payer
    operationalSolAccount = payer;
  })

  it("Is initialized!", async () => {
    const initData = {
      adminAuthority: payer,                       // 管理员地址
      validatorManagerAuthority: payer,            // 列表管理员
      minStake: new anchor.BN(1_000_000_000),             // 1 SOL（单位：lamports）
      rewardsFee: {                         // 例如 5% = 500 basis points
        basisPoints: 500
      },
      liqPool: {                            // LiqPoolInitializeData
        lpLiquidityTarget: new anchor.BN(50_000_000_000), // 50 SOL 目标流动性
        lpMaxFee: { basisPoints: 300 },     // 最大赎回费 3%
        lpMinFee: { basisPoints: 50 },     // 最小赎回费 0.5%
        lpTreasuryCut: { basisPoints: 200 } // 国库抽成 2%
      },
      additionalStakeRecordSpace: 0,       // 额外 stake_list 空间（字节）
      additionalValidatorRecordSpace: 0,    // 额外 validator_list 空间（字节）
      slotsForStakeDelta: new anchor.BN(3000),            // 每 3000 slots 允许一次 stake delta
      pauseAuthority: payer                       // 紧急暂停地址
    };

    // Add your test here.
    const tx = await program.methods
      .initialize(initData)
      .accounts({
        // ---- core ----
        payer,
        operationalSolAccount
      })
      .rpc();

    // ---------- 1. 读取链上账户 ----------
    const msolMintInfo = await provider.connection.getAccountInfo(msolPda);
    const lpMintInfo = await provider.connection.getAccountInfo(lpMintPda);
    const msolLegInfo = await provider.connection.getAccountInfo(msolLegtPda);

    if (!msolMintInfo || !lpMintInfo || !msolLegInfo) {
      throw new Error("One of the accounts does not exist");
    }

    const msolMintData = MintLayout.decode(msolMintInfo.data);
    const lpMintData = MintLayout.decode(lpMintInfo.data);
    const msolLegData = AccountLayout.decode(msolLegInfo.data);

    // ---------- 2. 预计算应当存在的 PDA ----------
    const [msolMintAuth] = PublicKey.findProgramAddressSync(
      [stakePoolConfigPda.toBuffer(), Buffer.from("st_mint")],
      program.programId
    );
    const [lpMintAuth] = PublicKey.findProgramAddressSync(
      [stakePoolConfigPda.toBuffer(), Buffer.from("liq_mint")],
      program.programId
    );
    const [msolLegAuth] = PublicKey.findProgramAddressSync(
      [stakePoolConfigPda.toBuffer(), Buffer.from("liq_st_sol_authority")],
      program.programId
    );
    const [expectedSolLeg] = PublicKey.findProgramAddressSync(
      [stakePoolConfigPda.toBuffer(), Buffer.from("liq_sol")],
      program.programId
    );

    // ---------- 3. 校验 msol_mint ----------
    if (!new PublicKey(msolMintData.mintAuthority).equals(msolMintAuth))
      throw new Error("msol_mint authority mismatch");
    if (msolMintData.supply !== BigInt(0))
      throw new Error("msol_mint supply not empty");
    if (msolMintData.freezeAuthorityOption !== 0)
      throw new Error("msol_mint still has freeze authority");

    // ---------- 4. 校验 lp_mint ----------
    if (!new PublicKey(lpMintData.mintAuthority).equals(lpMintAuth))
      throw new Error("lp_mint authority mismatch");
    if (!new anchor.BN(lpMintData.supply.toString()).eq(new anchor.BN(0)))
      throw new Error("lp_mint supply not empty");
    if (lpMintData.freezeAuthorityOption !== 0)
      throw new Error("lp_mint still has freeze authority");

    // ---------- 5. 校验 sol_leg PDA ----------
    if (!solLegtPda.equals(expectedSolLeg))
      throw new Error("sol_leg PDA address mismatch");

    // ---------- 6. 校验 msol_leg TokenAccount ----------
    if (!new PublicKey(msolLegData.mint).equals(msolPda))
      throw new Error("msol_leg mint 错误");
    if (!new PublicKey(msolLegData.owner).equals(msolLegAuth))
      throw new Error("msol_leg owner 错误");

    console.log("全部校验通过 ✅");

  });


});
