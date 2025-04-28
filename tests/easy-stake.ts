import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EasyStake } from "../target/types/easy_stake";
import { Keypair, PublicKey, SystemProgram, TransactionInstruction } from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  createInitializeMint2Instruction,
  createInitializeAccountInstruction,
  ACCOUNT_SIZE,
  MINT_SIZE,
  TOKEN_PROGRAM_ID
} from "@solana/spl-token";

describe("easy-stake", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.easyStake as Program<EasyStake>;

  const payer = provider.wallet.publicKey;

  const STAKE_POOL_CONFIG_SEED = Buffer.from("stake_pool");
  const RESERVE_SEED = Buffer.from("reserve");
  const STAKE_LIST_SEED = Buffer.from("stake_list");
  const VALIDATOR_LIST_SEED = Buffer.from("validator_list");
  const SOL_LEG_SEED = Buffer.from("liq_sol");
  const MSOL_LEG_SEED = Buffer.from("liq_st_sol_authority");
  const MSOL_MINT_SEED = Buffer.from("st_mint");
  const LP_MINT_SEED = Buffer.from("liq_mint");

  const additional_stake_record_space = 0;
  const additional_validator_record_space = 0;
  // stake_list大小
  const stake_list_space = 8 + 32 + 4 + 4 + 32 + 4 + 32 + 8 + 8 + 1;
  const validator_list_space = 8 + 32 + 4 + 4 + 32 + 4 + 32 + 8 + 4 + 8 + 4 + 1;

  // 所以公钥
  let stakePoolConfigPda: PublicKey;
  let stakePoolConfigBump: number;
  let reservePda: PublicKey;
  let reserveBump: number;
  let createReservetIx: TransactionInstruction;
  let solLegPda: PublicKey;
  let solLegBump: number;
  let solLegIx: TransactionInstruction;
  let stakeListPda: PublicKey;
  let stakeListBump: number;
  let createStakeListIx: TransactionInstruction;
  let validatorListPda: PublicKey;
  let validatorListBump: number;
  let createValidatorListIx: TransactionInstruction;
  let msolMintPda: PublicKey;
  let msolMintBump: number;
  let lpMintPda: PublicKey;
  let lpMintBum: number;
  let msolLegAta: PublicKey;
  let msolLegBump: number;

  before(async () => {
    const tokenRent = await provider.connection.getMinimumBalanceForRentExemption(
      ACCOUNT_SIZE
    );
    // 计算stakePoolConfig PDA
    [stakePoolConfigPda, stakePoolConfigBump] = PublicKey.findProgramAddressSync(
      [STAKE_POOL_CONFIG_SEED],
      program.programId
    );

    // 计算 reserve PDA
    [reservePda, reserveBump] = PublicKey.findProgramAddressSync(
        [stakePoolConfigPda.toBuffer(), RESERVE_SEED],
        program.programId
      );

    // 构建reserve 账户
    createReservetIx = SystemProgram.createAccount({
      fromPubkey: payer,
      newAccountPubkey: reservePda,
      space: 0,
      lamports: tokenRent,
      programId: program.programId
    });

    // 计算solLeg PDA
    [solLegPda, solLegBump] = PublicKey.findProgramAddressSync(
      [stakePoolConfigPda.toBuffer(), SOL_LEG_SEED],
      program.programId
    );

    // 构建solLeg 账户
    solLegIx = SystemProgram.createAccount({
      fromPubkey: payer,
      newAccountPubkey: solLegPda,
      space: 0,
      lamports: tokenRent,
      programId: program.programId
    });
    
    // 计算 stake_list PDA
    [stakeListPda, stakeListBump] = PublicKey.findProgramAddressSync(
        [stakePoolConfigPda.toBuffer(), STAKE_LIST_SEED],
        program.programId
      );
    // 创建 stake_list账户
    const stake_lamports = await provider.connection.getMinimumBalanceForRentExemption(stake_list_space);
    createStakeListIx = SystemProgram.createAccount({
      fromPubkey: payer,
      newAccountPubkey: stakeListPda,
      space: stake_list_space,
      lamports: stake_lamports,
      programId: program.programId
    });


    // 计算 validator_list PDA
    [validatorListPda, validatorListBump] = PublicKey.findProgramAddressSync(
        [stakePoolConfigPda.toBuffer(), VALIDATOR_LIST_SEED],
        program.programId
      );

    // 创建 validator_list 账户
    const validator_lamports = await provider.connection.getMinimumBalanceForRentExemption(validator_list_space);
    createValidatorListIx = SystemProgram.createAccount({
      fromPubkey: payer,
      newAccountPubkey: validatorListPda,
      space: validator_list_space,
      lamports: validator_lamports,
      programId: program.programId
    });

    // ---------- 创建 msol_mint & lp_mint ----------
    const payerKp = (provider.wallet as anchor.Wallet).payer as Keypair; // 现成的签名者
    const decimals = 9;
    const mintLamports = await provider.connection.getMinimumBalanceForRentExemption(
      MINT_SIZE
    );

    // 计算PAD
    [msolMintPda, msolMintBump] = PublicKey.findProgramAddressSync(
      [stakePoolConfigPda.toBuffer(), MSOL_MINT_SEED],
      program.programId
    );

    [lpMintPda, lpMintBum] = PublicKey.findProgramAddressSync(
      [stakePoolConfigPda.toBuffer(), LP_MINT_SEED],
      program.programId
    );

    [msolLegAta, msolLegBump] = PublicKey.findProgramAddressSync(
      [stakePoolConfigPda.toBuffer(), MSOL_LEG_SEED],
      program.programId
    );

    // 创建空账户
    const createMsolMintIx = SystemProgram.createAccount({
      fromPubkey: payer,
      newAccountPubkey: msolMintPda,
      space: MINT_SIZE,
      lamports: mintLamports,
      programId: TOKEN_PROGRAM_ID,
    });

    const createLpMintIx = SystemProgram.createAccount({
      fromPubkey: payer,
      newAccountPubkey: lpMintPda,
      space: MINT_SIZE,
      lamports: mintLamports,
      programId: TOKEN_PROGRAM_ID,
    });

    // 将账户初始化为 mint
    const initMsolMintIx = createInitializeMint2Instruction(
      msolMintPda,
      decimals,
      payer,              // mint_authority
      null,               // freeze_authority
      TOKEN_PROGRAM_ID
    );

    const initLpMintIx = createInitializeMint2Instruction(
      lpMintPda,
      decimals,
      payer,
      null,
      TOKEN_PROGRAM_ID
    );
    
    const createMsolLegIx = SystemProgram.createAccount({
      fromPubkey: payer,
      newAccountPubkey: msolLegAta,
      space: ACCOUNT_SIZE,
      lamports: tokenRent,
      programId: TOKEN_PROGRAM_ID,
    });
    const initMsolLegIx = createInitializeAccountInstruction(
      msolLegAta,
      msolMintPda,
      stakePoolConfigPda,        // owner = 全局 config PDA (off-curve)
      TOKEN_PROGRAM_ID
    );
    


  })

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
