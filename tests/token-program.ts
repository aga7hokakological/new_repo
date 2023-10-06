import { join } from "path";
import { readFileSync } from "fs";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import * as token from "@solana/spl-token";
import { TokenProgram } from "../target/types/token_program";
import { utf8 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { BN } from "bn.js";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

describe("token-program", async () => {
  // Configure the client to use the local cluster.
  let provider = anchor.AnchorProvider.env();
  let connection = provider.connection;
  anchor.setProvider(provider);

  const program = anchor.workspace.TokenProgram as Program<TokenProgram>;

  const WALLET_PATH = join(process.env["HOME"]!, ".config/solana/id.json");
  const admin = anchor.web3.Keypair.fromSecretKey(
    Buffer.from(JSON.parse(readFileSync(WALLET_PATH, { encoding: "utf-8" })))
  );

  const dex = anchor.web3.Keypair.generate();

  it("Is initialized!", async () => {
      // TOKEN0 mint 
  let token0mintPubkey = await token.createMint(
    connection,
    admin,
    admin.publicKey,
    null,
    9
  );
  let mint0Account = await token.getMint(connection, token0mintPubkey);
  // console.log("token0 mint pubkey::", mint0Account);

  // TOKEN1 mint 
  let token1mintPubkey = await token.createMint(
    connection,
    admin,
    admin.publicKey,
    null,
    9
  );
  let mint1Account = await token.getMint(connection, token1mintPubkey);
  console.log("token1 mint pubkey::", mint1Account.address.toString());

  const [dexPDA] = await anchor.web3.PublicKey.findProgramAddressSync([
    utf8.encode("dex"),
    token0mintPubkey.toBuffer(),
    token1mintPubkey.toBuffer(),
  ],
    program.programId
  );

  const [authorityPDA] = await anchor.web3.PublicKey.findProgramAddressSync([
    utf8.encode("authority"),
    dexPDA.toBuffer(),
  ],
    program.programId
  );


  // TOKEN LP mint 
  let tokenLpMintPubkey = await token.createMint(
    connection,
    admin,
    admin.publicKey,
    null,
    9
  );
  let tokenLpMintAccount = await token.getMint(connection, tokenLpMintPubkey);
  console.log("token1 mint pubkey::", tokenLpMintPubkey.toString());

  // ADMIN Token0 account
  let adminToken0Acc = await token.createAssociatedTokenAccount(
    connection,
    admin,
    token0mintPubkey,
    admin.publicKey
  );
  let adminToken0AccData = await token.getAccount(
    connection,
    adminToken0Acc
  );
  // console.log("TOKEN0 ACCOUNT DATA::", adminToken0AccData);

  // ADMIN Token1 account
  let adminToken1Acc = await token.createAssociatedTokenAccount(
    connection,
    admin,
    token1mintPubkey,
    admin.publicKey
  );
  let adminToken1AccData = await token.getAccount(
    connection,
    adminToken1Acc
  );
  console.log("TOKEN1 ACCOUNT DATA::", adminToken1AccData.address.toString());

    // ADMIN Token1 account
    let adminTokenLpAcc = await token.createAssociatedTokenAccount(
      connection,
      admin,
      tokenLpMintPubkey,
      admin.publicKey
    );
    let adminTokenLpAccData = await token.getAccount(
      connection,
      adminTokenLpAcc
    );
    console.log("TOKEN LP ACCOUNT DATA::", adminTokenLpAccData.address.toString());

  // Token0 mint
  let token0MintToAdmin = await token.mintToChecked(
    connection,
    admin,
    token0mintPubkey,
    adminToken0Acc,
    admin.publicKey,
    100e9,
    9
  )
  let token0Amount = await connection.getTokenAccountBalance(adminToken0Acc);
  console.log("AMOUNT0::", token0Amount.value.amount);
  // Token0 mint
  let token1MintToAdmin = await token.mintToChecked(
    connection,
    admin,
    token1mintPubkey,
    adminToken1Acc,
    admin.publicKey,
    100e9,
    9
  )
  let token1Amount = await connection.getTokenAccountBalance(adminToken1Acc);
  console.log("AMOUNT1::", token1Amount.value.amount);

  console.log("passed1")
    /// DEX ATA accounts
    const dexToken0Acc = await token.getOrCreateAssociatedTokenAccount(
      connection,
      admin,
      token0mintPubkey,
      dexPDA,
      true
    )

    const dexToken1Acc = await token.getOrCreateAssociatedTokenAccount(
      connection,
      admin,
      token1mintPubkey,
      dexPDA,
      true
    )

    const dexTokenLpAcc = await token.getOrCreateAssociatedTokenAccount(
      connection,
      admin,
      tokenLpMintPubkey,
      dexPDA,
      true
    )

    // let token1MintToDex = await token.mintToChecked(
    //   connection,
    //   admin,
    //   token1mintPubkey,
    //   dexToken1Acc.address,
    //   admin.publicKey,
    //   100e9,
    //   9
    // )
    // let token1AmtDex = await connection.getTokenAccountBalance(dexToken1Acc.address);
    // console.log("AMOUNT DEX::", token1AmtDex.value.amount);

    console.log("passed1")

    const dexTx = await program.methods.initializeDex().accounts({
      authority: authorityPDA,
      payer: admin.publicKey,
      dex: dexPDA,
      mintToken0: token0mintPubkey,
      mintToken1: token1mintPubkey,
      mintLp: tokenLpMintPubkey,
      accToken0: dexToken0Acc.address,
      accToken1: dexToken1Acc.address,
      accLp: dexTokenLpAcc.address,
      tokenProgram: token.TOKEN_PROGRAM_ID,
      // token1Program: token.TOKEN_PROGRAM_ID,
      // lpProgram: token.TOKEN_PROGRAM_ID,
    }).signers([admin]).rpc();
    console.log("tx::", dexTx)

      const addLiquidityTx = await program.methods.addLiquidity(new BN(50), new BN(50)).accounts({
        user: admin.publicKey,
        authority: admin.publicKey,
        dex: dexPDA,
        // mintToken0: token0mintPubkey,
        // mintToken1: token1mintPubkey,
        mintLp: tokenLpMintPubkey,
        userToken0: adminToken0Acc,
        userToken1: adminToken1Acc,
        userLp: adminTokenLpAcc,
        accToken0: dexToken0Acc.address,
        accToken1: dexToken1Acc.address,
        accLp: dexTokenLpAcc.address,
        tokenProgram: token.TOKEN_PROGRAM_ID,
      }).signers([admin]).rpc();
  let dexToken0Amt = await token.getAccount(connection, dexToken0Acc.address);
  console.log("AMOUNT0::", dexToken0Amt.amount.toString());
  let dexToken1Amt = await token.getAccount(connection, dexToken1Acc.address);
  console.log("AMOUNT1::", dexToken1Amt.amount.toString());
  let userToken1Amt = await token.getAccount(connection, adminToken1Acc);
  console.log("USER AMOUNT1::", userToken1Amt.amount.toString());

  const k = (await program.account.dex.fetch(dexPDA)).k;
  console.log(k.toString())

  // const removeLiquidityTx = await program.methods.removeLiquidity(new BN(2500)).accounts({
  //   user: admin.publicKey,
  //   authority: admin.publicKey,
  //   dex: dexPDA,
  //   mintLp: tokenLpMintPubkey,
  //   userToken0: adminToken0Acc,
  //   userToken1: adminToken1Acc,
  //   userLp: adminTokenLpAcc,
  //   accToken0: dexToken0Acc.address,
  //   accToken1: dexToken1Acc.address,
  //   accLp: dexTokenLpAcc.address,
  //   tokenProgram: token.TOKEN_PROGRAM_ID,
  // })

  const swapTx = await program.methods.swap(token0mintPubkey, new BN(10), new BN(10)).accounts({
    user: admin.publicKey,
    authority: admin.publicKey,
    dex: dexPDA,
    userToken0: adminToken0Acc,
    userToken1: adminToken1Acc,
    accToken0: dexToken0Acc.address,
    accToken1: dexToken1Acc.address,
    accLp: dexTokenLpAcc.address,
    tokenProgram: token.TOKEN_PROGRAM_ID,
  }).signers([admin]).rpc();
  // console.log("AMOUNT1::", dexToken1Amt.amount.toString());
  });
});
