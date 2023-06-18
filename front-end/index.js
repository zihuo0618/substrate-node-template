const { ApiPromise, WsProvider } = require('@polkadot/api');
const { StorageKey } = require('@polkadot/types');


async function main () {
  // 连接到节点
  const wsProvider = new WsProvider('ws://127.0.0.1:9944');
  const api = await ApiPromise.create({ provider: wsProvider });

  // 定义要读取的存储键（这里是一个示例）
  const key = '0x6e6f64652d74656d706c6174653a3a73746f726167653a3a0d000000';

  // 读取存储数据
  const storage = await api.rpc.offchain.localStorageGet(StorageKey.PERSISTENT,key);

  console.log("result: " + storage.toString()); // 打印读取到的存储数据
}

main().catch(console.error);

