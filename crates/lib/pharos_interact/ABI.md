# Invoice 合约 ABI 文档

本文档包含 Invoice 合约的主要功能 ABI，用于票据上链、查询和打包操作。

## 功能说明

### 1. 批量创建票据 (batchCreateInvoices)

批量创建票据到链上，支持一次性创建多个票据。

**注意事项：**

-   `timestamp` 和 `isValid` 会被合约自动设置
-   `dueDate` 必须大于当前时间
-   票据编号不能重复
-   金额不能为 0
-   只有债权人可以创建票据

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
                    "internalType": "string",
                    "name": "contractHash",
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
                    "internalType": "string",
                    "name": "tokenBatch",
                    "type": "string"
                },
                {
                    "internalType": "bool",
                    "name": "isCleared",
                    "type": "bool"
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

### 2. 创建票据打包批次 (createTokenBatch)

将多个票据打包成一个批次，用于后续发行 RBT Token。

**注意事项：**

-   批次中的所有票据必须属于同一个债权人和债务人
-   批次 ID 不能重复
-   期限和利率必须大于 0
-   最短期限不能大于最长期限

```json
{
    "inputs": [
        {
            "internalType": "string",
            "name": "_batchId",
            "type": "string"
        },
        {
            "internalType": "string[]",
            "name": "_invoiceNumbers",
            "type": "string[]"
        },
        {
            "internalType": "address",
            "name": "_stableToken",
            "type": "address"
        },
        {
            "internalType": "uint256",
            "name": "_minTerm",
            "type": "uint256"
        },
        {
            "internalType": "uint256",
            "name": "_maxTerm",
            "type": "uint256"
        },
        {
            "internalType": "uint256",
            "name": "_interestRate",
            "type": "uint256"
        }
    ],
    "name": "createTokenBatch",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
}
```

### 3. 确认发行票据批次 (confirmTokenBatchIssue)

债务人确认发行票据打包批次。

**注意事项：**

-   只有债务人可以确认发行
-   批次必须已创建且未发行
-   确认后批次状态变为已发行

```json
{
    "inputs": [
        {
            "internalType": "string",
            "name": "_batchId",
            "type": "string"
        }
    ],
    "name": "confirmTokenBatchIssue",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
}
```

### 4. 购买份额 (purchaseShares)

购买票据打包批次的份额。

**注意事项：**

-   批次必须已发行
-   需要足够的稳定币余额
-   购买金额会按比例分配给债权人和金库

```json
{
    "inputs": [
        {
            "internalType": "string",
            "name": "_batchId",
            "type": "string"
        },
        {
            "internalType": "uint256",
            "name": "_amount",
            "type": "uint256"
        }
    ],
    "name": "purchaseShares",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
}
```

### 5. 查询票据 (queryInvoices)

灵活查询票据信息，支持多种查询条件组合。

**参数说明：**

-   `batchId`: 批次 ID（可选）
-   `payer`: 债务人地址（可选）
-   `payee`: 债权人地址（可选）
-   `invoiceNumber`: 票据编号（可选）
-   `checkValid`: 是否只查询有效票据

```json
{
    "inputs": [
        {
            "components": [
                {
                    "internalType": "string",
                    "name": "batchId",
                    "type": "string"
                },
                {
                    "internalType": "address",
                    "name": "payer",
                    "type": "address"
                },
                {
                    "internalType": "address",
                    "name": "payee",
                    "type": "address"
                },
                {
                    "internalType": "string",
                    "name": "invoiceNumber",
                    "type": "string"
                },
                {
                    "internalType": "bool",
                    "name": "checkValid",
                    "type": "bool"
                }
            ],
            "internalType": "struct Invoice.QueryParams",
            "name": "params",
            "type": "tuple"
        }
    ],
    "name": "queryInvoices",
    "outputs": [
        {
            "components": [
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
                            "internalType": "string",
                            "name": "contractHash",
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
                            "internalType": "string",
                            "name": "tokenBatch",
                            "type": "string"
                        },
                        {
                            "internalType": "bool",
                            "name": "isCleared",
                            "type": "bool"
                        },
                        {
                            "internalType": "bool",
                            "name": "isValid",
                            "type": "bool"
                        }
                    ],
                    "internalType": "struct Invoice.InvoiceData[]",
                    "name": "invoices",
                    "type": "tuple[]"
                },
                {
                    "internalType": "uint256",
                    "name": "total",
                    "type": "uint256"
                }
            ],
            "internalType": "struct Invoice.QueryResult",
            "name": "",
            "type": "tuple"
        }
    ],
    "stateMutability": "view",
    "type": "function"
}
```

### 6. 获取批次信息 (getTokenBatch)

查询票据打包批次的详细信息。

```json
{
    "inputs": [
        {
            "internalType": "string",
            "name": "_batchId",
            "type": "string"
        }
    ],
    "name": "getTokenBatch",
    "outputs": [
        {
            "components": [
                {
                    "internalType": "string",
                    "name": "batchId",
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
                    "internalType": "address",
                    "name": "stableToken",
                    "type": "address"
                },
                {
                    "internalType": "uint256",
                    "name": "minTerm",
                    "type": "uint256"
                },
                {
                    "internalType": "uint256",
                    "name": "maxTerm",
                    "type": "uint256"
                },
                {
                    "internalType": "uint256",
                    "name": "interestRate",
                    "type": "uint256"
                },
                {
                    "internalType": "uint256",
                    "name": "totalAmount",
                    "type": "uint256"
                },
                {
                    "internalType": "uint256",
                    "name": "issueDate",
                    "type": "uint256"
                },
                {
                    "internalType": "bool",
                    "name": "isSigned",
                    "type": "bool"
                },
                {
                    "internalType": "bool",
                    "name": "isIssued",
                    "type": "bool"
                },
                {
                    "internalType": "string[]",
                    "name": "invoiceNumbers",
                    "type": "string[]"
                }
            ],
            "internalType": "struct Invoice.InvoiceTokenBatch",
            "name": "",
            "type": "tuple"
        }
    ],
    "stateMutability": "view",
    "type": "function"
}
```

### 7. 获取用户批次列表 (getUserBatches)

查询指定用户的所有票据打包批次 ID。

```json
{
    "inputs": [
        {
            "internalType": "address",
            "name": "_user",
            "type": "address"
        }
    ],
    "name": "getUserBatches",
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

### 8. 作废票据 (invalidateInvoice)

管理员作废指定票据。

**注意事项：**

-   只有合约所有者可以作废票据
-   票据必须存在且有效

```json
{
    "inputs": [
        {
            "internalType": "string",
            "name": "_invoiceNumber",
            "type": "string"
        }
    ],
    "name": "invalidateInvoice",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
}
```

### 9. 暂停/恢复合约 (pause/unpause)

管理员暂停或恢复合约功能。

**注意事项：**

-   只有合约所有者可以暂停/恢复合约
-   暂停状态下，除查询外的所有功能都将被禁用

```json
{
    "inputs": [],
    "name": "pause",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
},
{
    "inputs": [],
    "name": "unpause",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
}
```
