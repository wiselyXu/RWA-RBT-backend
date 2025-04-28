# RWA 应收票据份额化平台 (RWA Receivable Bill Tokenization Platform)

## 1. 项目简介 (Project Overview)

本项目旨在构建一个基于区块链的平台，将企业间的应收票据（应收账款）进行数字化和份额化处理。通过将多笔应收票据打包，并发行对应的数字份额代币（RBT - Receivable Bill Token），为市场提供一种新的投资渠道，同时帮助企业（债权人）加快资金周转，并为债务人提供一个积累信用的机制。

**核心目标:**

*   **加速资金流动:** 帮助持有应收票据的债权人提前获得资金。
*   **提升企业信用:** 为按时还款的债务人提供信用证明。
*   **创造投资机会:** 为投资者提供基于实体经济债权的投资标的。
*   **降低坏账风险:** 通过市场化运作和透明度，潜在地降低坏账率。

**主要参与方:**

*   **债权人 (Creditor):** 持有应收票据的企业。
*   **债务人 (Debtor):** 需要支付应收票据款项的企业。
*   **投资者 (Investor):** 购买 RBT 份额以获取投资收益的个人或机构。
*   **平台 (Platform):** 提供技术基础设施、审核、撮合等服务（包括智能合约和后端系统）。

## 2. 核心概念 (Key Concepts)

*   **应收票据 (Receivable Bill):** 企业间因商品或服务交易产生的未来收款权利凭证。
*   **份额化/代币化 (Tokenization):** 将一组应收票据的总价值，按照一定规则（如 1:1 锚定稳定币价值）映射为可在区块链上发行、流转、记录的数字代币（RBT）。
*   **RBT (Receivable Bill Token):** 代表特定一批次应收票据价值份额的同质化代币。其价值会随时间（计息）和债务人还款情况而变动。
*   **批次 (Batch):** 一组被共同打包、用于发行同一 RBT 的应收票据。通常要求同一批次内的票据具有相同的债权人和债务人。
*   **委托钱包 (Escrow/Delegated Wallet):** 用于初始存放 minted RBT 份额、接收投资者稳定币、向债权人按比例支付、接收债务人还款、向投资者分配稳定币的合约控制或平台控制的地址。

## 3. 业务流程 / 核心功能 (Business Workflow / Core Features)

平台的核心运作流程包含以下步骤：

**3.1. 企业入驻与认证 (Enterprise Onboarding & KYC - Off-chain)**
*   债权人和债务人需要在平台注册并完成认证（提供企业信息、资质证明等）。
*   平台进行审核（简化模式下可跳过）。

**3.2. 票据上链/登记 (Invoice Onboarding)**
*   债权人提交应收票据信息（票据编号、金额、债务人地址、票据/合同影印件 IPFS Hash 等）。
*   理想情况下需要债权人和债务人共同签名确认票据的真实性和有效性（简化模式下可先由债权人登记）。

**3.3. 票据打包与 RBT 发行 (Invoice Packaging & RBT Issuance)**
*   债权人选择同一债务人的多张未处理票据进行打包。
*   平台（或智能合约）进行预检查。
*   设定发行参数：接受的还款稳定币种、期限、年化利率、违约利率等。
*   需要债权人和债务人双方签名确认发行。
*   智能合约根据票据总额计算并 `mint` 相应数量的 RBT 代币。
*   新发行的 RBT 存入委托钱包地址。

**3.4. 市场交易 / 份额购买 (Marketplace Trading / RBT Purchase)**
*   平台展示可供购买的 RBT 批次信息。
*   投资者使用指定的稳定币购买 RBT 份额。
*   **资金/份额流转:** 投资者的稳定币 -> 委托钱包地址 -> (部分)债权人地址；委托钱包地址的 RBT -> 投资者地址。

**3.5. 利息计算与分配 (Interest Accrual & Distribution)**
*   智能合约（或由后端触发）按约定频率（如每日）计算当前批次 RBT 的应计利息。
*   合约 `mint` 新的 RBT 代币（代表利息）。
*   新增的 RBT 份额按比例分配给当前的 RBT 持有者（直接分配或记录待领取）。

**3.6. 债务人还款 (Debtor Repayment)**
*   债务人使用约定的稳定币进行还款。
*   **资金/份额流转:** 债务人的稳定币 -> 委托钱包地址；合约按比例 `burn` 各持有者的 RBT；合约按比例将稳定币分配给 RBT 持有者（或记录待领）。

**3.7. 清算与凭证发放 (Settlement & NFT Issuance)**
*   当债务人偿还了所有本金和利息，对应批次的 RBT 完成生命周期。
*   智能合约可以发放一个纪念性的 NFT 给债权人和债务人。

**3.8. 查询与账单 (Queries & Statements)**
*   平台应提供界面供各方查询票据、RBT 批次、交易、个人账单等信息。

## 4. 技术选型 (Technology Stack)

*   **前端 (Frontend):**
    *   **框架/库 (Framework/Library):** React
    *   **说明:** 选用 React 构建交互式、组件化的用户界面，拥有庞大的社区支持和丰富的生态系统。
*   **后端 (Backend):**
    *   **语言 (Language):** Rust
    *   **Web 框架 (Web Framework):** (待定，可选如 Actix Web, Axum, Rocket)
    *   **ORM:** SeaORM
    *   **说明:** Rust 提供高性能和内存安全，适合构建可靠的后端服务。SeaORM 是一个基于 Rust 的异步动态 ORM，有助于数据库交互。
*   **区块链/链端 (Blockchain/On-chain):**
    *   **链平台 (Platform):** Pharos Chain
    *   **智能合约语言 (Smart Contract Language):** Rust (利用 Stylus SDK)
    *   **说明:** 在 Pharos Chain 上使用 Rust 编写智能合约，可以利用 WASM 的执行效率和 Rust 的安全性优势。Stylus SDK 使得用 Rust (及其他 WASM 目标语言) 开发与 EVM 兼容的智能合约成为可能。
*   **数据库 (Database):** (待定，可选如 PostgreSQL, MySQL)
*   **文件存储 (File Storage):** IPFS (用于存储票据/合同等大文件)

## 5. 当前技术实现状态 (Current Technical Implementation Status)

本仓库 (`pharos-invoice-interact`) 目前包含一个**链下 Rust 应用程序**，用于与已部署在 Pharos 链上的 `InvoiceContract` 智能合约进行交互。

*   **库 (`src/lib.rs`):**
    *   使用 `ethers-rs` 库。
    *   通过 ABI (`invoice_abi.json`) 生成合约绑定。
    *   封装了与合约交互的函数，如 `initialize_contract_from_env`, `batch_create_invoices`, `get_invoice`, `get_user_invoices`。
    *   包含单元测试 (需要 `.env` 配置和网络连接)。
*   **主程序 (`src/main.rs`):**
    *   使用 `tokio` 异步运行时。
    *   从 `.env` 文件读取 RPC URL、合约地址、私钥。
    *   初始化合约连接。
    *   提供调用 `batch_create_invoices` 创建测试票据，然后调用 `get_invoice` 和 `get_user_invoices` 进行验证的示例。
*   **配置文件:**
    *   `Cargo.toml`: 定义项目依赖 (`ethers-rs`, `tokio`, `serde`, `dotenv`, `rand` 等)。
    *   `.env` (示例，需自行创建): 存储链连接信息。
    *   `invoice_abi.json`: `InvoiceContract` 的 ABI 文件。

**注意:** 当前代码是用于**调用**链上合约的客户端，而不是合约本身的实现。

## 6. 如何运行 (Setup & Running)

1.  **安装 Rust:** 确保已安装 Rust 环境 (参考: <https://www.rust-lang.org/tools/install>)。
2.  **克隆仓库:** `git clone <repository-url>`
3.  **进入目录:** `cd pharos-invoice-interact`
4.  **创建 `.env` 文件:** 在项目根目录创建 `.env` 文件，并填入以下内容 (替换为你的实际值):
    ```dotenv
    PHAROS_RPC_URL=https://your-pharos-rpc-endpoint
    INVOICE_CONTRACT_ADDRESS=0xYourDeployedInvoiceContractAddress
    SIGNER_PRIVATE_KEY=0xyourPrivateKeyForSendingTransactions
    ```
5.  **编译:** `cargo build`
6.  **运行主程序:** `cargo run` (这将执行 `src/main.rs` 中的示例交互逻辑)
7.  **运行测试:** `cargo test` (部分测试可能需要网络和 `.env` 配置，并且默认被 `#[ignore]`，运行它们需要 `cargo test -- --ignored`)

## 7. 未来方向 (Future Work)

*   实现完整的智能合约逻辑 (票据池、RBT 代币、计息、还款、NFT 等)。
*   开发后端服务处理链下逻辑、数据库存储、API 服务。
*   开发前端用户界面。
*   实现更复杂的业务逻辑（如部分还款、多币种支持、风险控制）。
*   集成 IPFS 存储文件。
*   添加更全面的错误处理和日志记录。
*   安全性审计。



cargo stylus deploy --endpoint  https://devnet.dplabs-internal.com --private-key 0fbea5137261a5af747cdcdb9799bef06476eac225de4d818120bcfd7e096c14 
 

