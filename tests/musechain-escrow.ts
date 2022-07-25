import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { MusechainEscrow } from "../target/types/musechain_escrow";
const assert = require("assert");
const spl = require("@solana/spl-token");

describe("musechain-escrow", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MusechainEscrow as Program<MusechainEscrow>;

  let buyer = anchor.web3.Keypair.generate();
  let seller = anchor.web3.Keypair.generate();

  let buyerTokenAccount;
  let sellerTokenAccount;
  let tokenMint;

  const sellingPrice = 500000000;

  it("fund these accounts", async () => {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(buyer.publicKey, 10000000000),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(seller.publicKey, 10000000000),
      "confirmed"
    );

    let buyerUserBalance = await provider.connection.getBalance(
      buyer.publicKey
    );
    let sellerUserBalance = await provider.connection.getBalance(
      seller.publicKey
    );
    assert.strictEqual(10000000000, buyerUserBalance);
    assert.strictEqual(10000000000, sellerUserBalance);
  });

  it("create a token mint and mint tokens to owner wallet", async () => {
    tokenMint = await spl.createMint(
      provider.connection,
      seller,
      seller.publicKey,
      seller.publicKey,
      6
    );

    buyerTokenAccount = await spl.createAccount(
      provider.connection,
      buyer,
      tokenMint,
      buyer.publicKey
    );
    sellerTokenAccount = await spl.createAccount(
      provider.connection,
      seller,
      tokenMint,
      seller.publicKey
    );

    await spl.mintTo(
      provider.connection,
      seller,
      tokenMint,
      sellerTokenAccount,
      seller.publicKey,
      1,
      [seller]
    );

    const sellerTokenBalance = await spl.getAccount(
      provider.connection,
      sellerTokenAccount 
    );
    assert.equal(sellerTokenBalance.amount, 1);
  });

  it("Initialize the PDA ", async () => {
    // Add your test here.
    const [lockAccountPDA, lockAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("owner"), seller.publicKey.toBuffer(), tokenMint.toBuffer()],
      program.programId
    )

    const [escrowTokenAccountPDA, escrowTokenAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("token"), seller.publicKey.toBuffer(), tokenMint.toBuffer()],
      program.programId
    )

    const tx = await program.methods.initialize().accounts({
      lockAccount: lockAccountPDA,
      escrowTokenAccount: escrowTokenAccountPDA,
      owner: seller.publicKey,
      mintAddress: tokenMint,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY
    }).signers([seller]).rpc();

    const state = await program.account.lockAccount.fetch(lockAccountPDA);   

    assert.equal(state.owner.toBase58(), seller.publicKey.toBase58());
    assert.equal(state.mintAddress.toBase58() ,tokenMint.toBase58());

  });

  it("List the Token", async () => {
    const [lockAccountPDA, lockAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("owner"), seller.publicKey.toBuffer(), tokenMint.toBuffer()],
      program.programId
    )

    const [escrowTokenAccountPDA, escrowTokenAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("token"), seller.publicKey.toBuffer(), tokenMint.toBuffer()],
      program.programId
    )

    const tx = await program.methods.listNft(lockAccountBump, escrowTokenAccountBump, new anchor.BN(sellingPrice)).accounts({
      lockAccount: lockAccountPDA,
      escrowTokenAccount: escrowTokenAccountPDA,
      owner: seller.publicKey,
      nftAccount: sellerTokenAccount,
      mintAddress: tokenMint,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY
    }).signers([seller]).rpc();

    const sellerTokenAccountBalance = await spl.getAccount(provider.connection, buyerTokenAccount);
    const escrowTokenAccountBalance = await spl.getAccount(provider.connection, escrowTokenAccountPDA);

    assert.equal(sellerTokenAccountBalance.amount, 0)
    assert.equal(escrowTokenAccountBalance.amount, 1)

  });

  it("Buy the token", async () => {
    const [lockAccountPDA, lockAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("owner"), seller.publicKey.toBuffer(), tokenMint.toBuffer()],
      program.programId
    )

    const [escrowTokenAccountPDA, escrowTokenAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("token"), seller.publicKey.toBuffer(), tokenMint.toBuffer()],
      program.programId
    )

    const buyerBalance = await provider.connection.getBalance(buyer.publicKey);
    const sellerBalance = await provider.connection.getBalance(seller.publicKey);

    const tx = await program.methods.buy(lockAccountBump, escrowTokenAccountBump).accounts({
      lockAccount: lockAccountPDA,
      escrowTokenAccount: escrowTokenAccountPDA,
      buyer: buyer.publicKey,
      seller: seller.publicKey,
      buyerTokenAccount: buyerTokenAccount,
      mintAddress: tokenMint,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY
    }).signers([buyer]).rpc();

    const buyerBalanceAfter = await provider.connection.getBalance(buyer.publicKey);
    const sellerBalanceAfter = await provider.connection.getBalance(seller.publicKey);

    assert.equal(buyerBalance - buyerBalanceAfter, sellingPrice)
    assert.equal(sellerBalanceAfter - sellerBalance, sellingPrice)

    const buyerTokenBalance = await spl.getAccount(provider.connection, buyerTokenAccount);
    const escrowTokenBalance = await spl.getAccount(provider.connection, escrowTokenAccountPDA);

    assert.equal(buyerTokenBalance.amount, 1);
    assert.equal(escrowTokenBalance.amount, 0);

  });

  it("seller Buys back the token", async () => {
    const [lockAccountPDA, lockAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("owner"), seller.publicKey.toBuffer(), tokenMint.toBuffer()],
      program.programId
    )

    const [escrowTokenAccountPDA, escrowTokenAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("token"), seller.publicKey.toBuffer(), tokenMint.toBuffer()],
      program.programId
    )

    const buyerBalance = await provider.connection.getBalance(buyer.publicKey);
    const sellerBalance = await provider.connection.getBalance(seller.publicKey);

    const tx = await program.methods.buy(lockAccountBump, escrowTokenAccountBump).accounts({
      lockAccount: lockAccountPDA,
      escrowTokenAccount: escrowTokenAccountPDA,
      buyer: seller.publicKey,
      seller: seller.publicKey,
      buyerTokenAccount: sellerTokenAccount,
      mintAddress: tokenMint,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY
    }).signers([seller]).rpc();

    const buyerBalanceAfter = await provider.connection.getBalance(buyer.publicKey);
    const sellerBalanceAfter = await provider.connection.getBalance(seller.publicKey);

    assert.equal(buyerBalance - buyerBalanceAfter, 0)
    assert.equal(sellerBalanceAfter - sellerBalance, 0)

    const sellerTokenBalance = await spl.getAccount(provider.connection, sellerTokenAccount);
    const escrowTokenBalance = await spl.getAccount(provider.connection, escrowTokenAccountPDA);

    assert.equal(sellerTokenBalance.amount, 1);
    assert.equal(escrowTokenBalance.amount, 0);

  });

});
