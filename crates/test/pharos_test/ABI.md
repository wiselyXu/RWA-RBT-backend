# Invoice 合约 ABI 文档

本文档包含 Invoice 合约的主要功能 ABI，用于票据上链和查询操作。

## 功能说明

### 1. 批量创建票据 (batchCreateInvoices)

批量创建票据到链上，支持一次性创建多个票据。

**注意事项：**

-   `timestamp` 和 `isValid` 会被合约重写
-   `dueDate` 必须大于当前时间
-   票据编号不能重复
-   金额不能为 0

```json
{
    "inputs": [
        {
            "components": [
                {
                    "internalType": "string",
                    "name": "invoiceNumber",
                    "type": "string"
                },
                {
                    "internalType": "address",
                    "name": "payee",
                    "type": "address"
                },
                {
                    "internalType": "address",
                    "name": "payer",
                    "type": "address"
                },
                {
                    "internalType": "uint256",
                    "name": "amount",
                    "type": "uint256"
                },
                {
                    "internalType": "string",
                    "name": "ipfsHash",
                    "type": "string"
                },
                {
                    "internalType": "uint256",
                    "name": "timestamp",
                    "type": "uint256"
                },
                {
                    "internalType": "uint256",
                    "name": "dueDate",
                    "type": "uint256"
                },
                {
                    "internalType": "bool",
                    "name": "isValid",
                    "type": "bool"
                }
            ],
            "internalType": "struct Invoice.InvoiceData[]",
            "name": "_invoices",
            "type": "tuple[]"
        }
    ],
    "name": "batchCreateInvoices",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
}
```

### 2. 查询单个票据 (getInvoice)

根据票据编号查询票据详细信息，可选择是否只查询有效票据。

**参数说明：**

-   `_invoiceNumber`: 票据编号
-   `_checkValid`: 是否只查询有效票据（true: 只返回有效票据，false: 返回所有票据）

```json
{
    "inputs": [
        {
            "internalType": "string",
            "name": "_invoiceNumber",
            "type": "string"
        },
        {
            "internalType": "bool",
            "name": "_checkValid",
            "type": "bool"
        }
    ],
    "name": "getInvoice",
    "outputs": [
        {
            "components": [
                {
                    "internalType": "string",
                    "name": "invoiceNumber",
                    "type": "string"
                },
                {
                    "internalType": "address",
                    "name": "payee",
                    "type": "address"
                },
                {
                    "internalType": "address",
                    "name": "payer",
                    "type": "address"
                },
                {
                    "internalType": "uint256",
                    "name": "amount",
                    "type": "uint256"
                },
                {
                    "internalType": "string",
                    "name": "ipfsHash",
                    "type": "string"
                },
                {
                    "internalType": "uint256",
                    "name": "timestamp",
                    "type": "uint256"
                },
                {
                    "internalType": "uint256",
                    "name": "dueDate",
                    "type": "uint256"
                },
                {
                    "internalType": "bool",
                    "name": "isValid",
                    "type": "bool"
                }
            ],
            "internalType": "struct Invoice.InvoiceData",
            "name": "",
            "type": "tuple"
        }
    ],
    "stateMutability": "view",
    "type": "function"
}
```

### 3. 查询用户票据列表 (getUserInvoices)

查询指定用户地址下的所有票据编号。

**参数说明：**

-   `_user`: 用户地址（通常是收款方地址）

```json
{
    "inputs": [
        {
            "internalType": "address",
            "name": "_user",
            "type": "address"
        }
    ],
    "name": "getUserInvoices",
    "outputs": [
        {
            "internalType": "string[]",
            "name": "",
            "type": "string[]"
        }
    ],
    "stateMutability": "view",
    "type": "function"
}
```

### 4. 票据映射查询 (invoices)

直接通过票据编号查询票据信息的映射。

**注意：** 推荐使用 `getInvoice` 函数代替直接查询映射。

```json
{
    "inputs": [
        {
            "internalType": "string",
            "name": "",
            "type": "string"
        }
    ],
    "name": "invoices",
    "outputs": [
        {
            "internalType": "string",
            "name": "invoiceNumber",
            "type": "string"
        },
        {
            "internalType": "address",
            "name": "payee",
            "type": "address"
        },
        {
            "internalType": "address",
            "name": "payer",
            "type": "address"
        },
        {
            "internalType": "uint256",
            "name": "amount",
            "type": "uint256"
        },
        {
            "internalType": "string",
            "name": "ipfsHash",
            "type": "string"
        },
        {
            "internalType": "uint256",
            "name": "timestamp",
            "type": "uint256"
        },
        {
            "internalType": "uint256",
            "name": "dueDate",
            "type": "uint256"
        },
        {
            "internalType": "bool",
            "name": "isValid",
            "type": "bool"
        }
    ],
    "stateMutability": "view",
    "type": "function"
}
```

