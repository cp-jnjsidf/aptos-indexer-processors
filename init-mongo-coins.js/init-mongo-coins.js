// Load required modules
const fs = require('fs');

// Record the start time
const startTime = new Date();

// URL prefixes for different sources
const URL_PREFIXES = {
  okx: "https://www.okx.com/priapi/v1/dx/market/toastTokenCheck?chainId=637&tokenAddress="
};

// DEX configuration with data paths
const DEX_CONFIG = {
  okx: "tokenLogoUrl=data.toTokenInfo.tokenLogoUrl;decimals=data.toTokenInfo.decimals;tokenName=data.toTokenInfo.tokenName;tokenSymbol=data.toTokenInfo.tokenSymbol"
};

// Coin configurations
const DEX_COIN_CONFIGS = {
    "okx_APT" : "0x1::aptos_coin::AptosCoin",
    "okx_sthAPT" : "0xfaf4e633ae9eb31366c9ca24214231760926576c7b625313b3688b5e900731f6::staking::StakedThalaAPT",
    "okx_thAPT" : "0xfaf4e633ae9eb31366c9ca24214231760926576c7b625313b3688b5e900731f6::staking::ThalaAPT",
    "okx_zUSDC" : "0xf22bede237a07e121b56d91a491eb7bcdfd1f5907926a9e58338f964a01b17fa::asset::USDC",
    "okx_MOV" : "0x6f986d146e4a90b828d8c12c14b6f4e003fdff11a8eecceceb63744363eaac01::mod_coin::MOD",
    "okx_amAPT" : "0x111ae3e5bc816a5e63c2da97d0aa3886519e0cd5e4b046659fa35796bd11542a::amapt_token::AmnisApt",
    "okx_whUSDC" : "0x5e156f1207d0ebfa19a9eeff00d62a282278fb8719f4fab3a586a0a2c0fffbea::coin::T",
    "okx_zWETH" : "0xf22bede237a07e121b56d91a491eb7bcdfd1f5907926a9e58338f964a01b17fa::asset::WETH",
    "okx_stAPT-Staked-Aptos-Coin" : "0x111ae3e5bc816a5e63c2da97d0aa3886519e0cd5e4b046659fa35796bd11542a::stapt_token::StakedApt",
    "okx_THL" : "0x7fd500c11216f0fe3095d0c4b8aa4d64a4e2e04f83758462f2b127255643615::thl_coin::THL",
    "okx_zUSDT" : "0xf22bede237a07e121b56d91a491eb7bcdfd1f5907926a9e58338f964a01b17fa::asset::USDT",
    "okx_CHEWY" : "0xc26a8eda1c3ab69a157815183ddda88c89d6758ee491dd1647a70af2907ce074::coin::Chewy",
    "okx_CAKE" : "0x159df6b7689437016108a019fd5bef736bac692b6d4a1f10c941f6fbb9a74ca6::oft::CakeOFT",
    "okx_USDC" : "0x5e156f1207d0ebfa19a9eeff00d62a282278fb8719f4fab3a586a0a2c0fffbea::coin::T",
    "okx_GUI" : "0xe4ccb6d39136469f376242c31b34d10515c8eaaa38092f804db8e08a8f53c5b2::assets_v1::EchoCoin002",
    "okx_SOL" : "0xdd89c0e695df0692205912fb69fc290418bed0dbe6e4573d744a6d5e6bab6c13::coin::T",
    "okx_MOVE" : "0x27fafcc4e39daac97556af8a803dbb52bcb03f0821898dc845ac54225b9793eb::move_coin::MoveCoin",
    "okx_USDT" : "0xa2eda21a58856fda86451436513b867c97eecb4ba099da5775520e0f7492e852::coin::T",
    "okx_tAPT" : "0x84d7aeef42d38a5ffc3ccef853e1b82e4958659d16a7de736a29c55fbbeb0114::staked_aptos_coin::StakedAptosCoin",
    "okx_WETH" : "0xcc8a89c8dce9693d354449f1f73e60e14e347417854f029db5bc8e7454008abb::coin::T",
    "okx_WBTC" : "0xae478ff7d83ed072dbc5e264250e67ef58f57c99d89b447efd8a0a2e8b2be76e::coin::T",
    "okx_MOJO" : "0x881ac202b1f1e6ad4efcff7a1d0579411533f2502417a19211cfc49751ddb5f4::coin::MOJO",
    //"okx_MOVER" : "0x14b0ef0ec69f346bea3dfa0c5a8c3942fb05c08760059948f9f24c02cd0d4fd8::mover_token::Mover",
    //"okx_APTSWAP" : "0x5c738a5dfa343bee927c39ebe85b0ceb95fdb5ee5b323c95559614f5a77c47cf::AptSwap::AptSwapGovernance",
    "okx_stAPT-DITTO-STAKED-APTOS" : "0xd11107bdf0d6d7040c6c0bfbdecb6545191fdf13e8d8d259952f53e1713f61b5::staked_coin::StakedAptos",
    "okx_ceUSDC" : "0x8d87a65ba30e09357fa2edea2c80dbac296e5dec2b18287113500b902942929d::celer_coin_manager::UsdcCoin",
    //"okx_ceWETH" : "0x8d87a65ba30e09357fa2edea2c80dbac296e5dec2b18287113500b902942929d::celer_coin_manager::WethCoin",
    //"okx_DONK" : "0xe88ae9670071da40a9a6b1d97aab8f6f1898fdc3b8f1c1038b492dfad738448b::coin::Donk",
    //"okx_SPRING" : "0x7bdeaba6f037caf06bb5b2d57df9ee03a07e2a9df45b338ef3deb44d16c01d10::spring_coin::Spring_Coin",
    //"okx_SLT" : "0x8b2df69c9766e18486c37e3cfc53c6ce6e9aa58bbc606a8a0a219f24cf9eafc1::sui_launch_token::SuiLaunchToken",
    "okx_ceUSDT" : "0x8d87a65ba30e09357fa2edea2c80dbac296e5dec2b18287113500b902942929d::celer_coin_manager::UsdtCoin",
    //"okx_DDOS" : "0x66398cf97d29fd3825f65b37cb2773268e5438d37e20777e6a98261da0cf1f1e::ddos_coin::DDOS_COIN",
    //"okx_wTBT" : "0xd916a950d4c1279df4aa0d6f32011842dc5c633a45c11ac5019232c159d115bb::coin::T",
    //"okx_USDCso" : "0xc91d826e29a3183eb3b6f6aa3a722089fdffb8e9642b94c5fcd4c48d035c0080::coin::T",
    //"okx_GARI" : "0x4def3d3dee27308886f0a3611dd161ce34f977a9a5de4e80b237225923492a2a::coin::T",
    //"okx_EVA" : "0x1fc2f33ab6b624e3e632ba861b755fd8e61d2c2e6cf8292e415880b4c198224d::apt20::EVA",
    //"okx_ABEL" : "0x7c0322595a73b3fc53bb166f5783470afeb1ed9f46d1176db62139991505dc61::abel_coin::AbelCoin",
    "okx_MAU" : "0x83b619e2d9e6e10d15ed4b714111a4cd9526c1c2ae0eec4b252a619d3e8bdda3::MAU::MAU",
    "okx_ceBNB" : "0x8d87a65ba30e09357fa2edea2c80dbac296e5dec2b18287113500b902942929d::celer_coin_manager::BnbCoin",
    //"okx_ETERN" : "0x25a64579760a4c64be0d692327786a6375ec80740152851490cfd0b53604cf95::coin::ETERN",
    //"okx_MBX" : "0x665d06fcd9c94430099f82973f2a5e5f13142e42fa172e72ce14f51a64bd8ad9::coin_mbx::MBX",
    "okx_APD" : "0xcc78307c77f1c2c0fdfee17269bfca7876a0b35438c3442417480c0d5c370fbc::AptopadCoin::APD",
    //"okx_multiUSDC" : "0xd6d6372c8bde72a7ab825c00b9edd35e643fb94a61c55d9d94a9db3010098548::USDC::Coin",
    "okx_DLC" : "0x84edd115c901709ef28f3cb66a82264ba91bfd24789500b6fd34ab9e8888e272::coin::DLC",
    "okx_USDA" : "0x1000000fa32d122c18a6a31c009ce5e71674f22d06a581bb0a15575e6addadcc::usda::USDA",
    //"okx_ALI" : "0x27975005fd8b836a905dc7f81c51f89e76091a4d0c4d694265f6eae0c05cb400::proton_a5d::PROTON_E54",
    //"okx_ANI" : "0x16fe2df00ea7dde4a63409201f7f4e536bde7bb7335526a35d05111e68aa322c::AnimeCoin::ANI",
    "okx_APTOGE" : "0x5c738a5dfa343bee927c39ebe85b0ceb95fdb5ee5b323c95559614f5a77c47cf::Aptoge::Aptoge",
    "okx_APTS" : "0xc71d94c49826b7d81d740d5bfb80b001a356198ed7b8005ae24ccedff82b299c::bridge::APTS",
    //"okx_BLADEE" : "0x8235f05ea1901e682bc09b3be93eba0727e94c020ccb0e57074843315c675521::BLADEEWIFHAT::BLADEEWIFHAT",
    //"okx_BUSD" : "0xccc9620d38c4f3991fa68a03ad98ef3735f18d04717cb75d7a1300dd8a7eed75::coin::T",
    //"okx_DOOT" : "0x9906c12b3b7a12721b9dddf23e6dd5ff5dfc93c5241dada855780758b01fedd3::DOOT_SKELETON::DOOT_SKELETON",
    //"okx_WBNB" : "0x6312bc0a484bc4e37013befc9949df2d7c8a78e01c6fe14a34018449d136ba86::coin::T",
    //"okx_ceDAI" : "0x8d87a65ba30e09357fa2edea2c80dbac296e5dec2b18287113500b902942929d::celer_coin_manager::DaiCoin",
    //"okx_USDCbs" : "0x79a6ed7a0607fdad2d18d67d1a0e552d4b09ebce5951f1e5c851732c02437595::coin::T",
    //"okx_USDCpo" : "0xc7160b1c2415d19a88add188ec726e62aab0045f0aed798106a2ef2994a9101e::coin::T",
    //"okx_ceBUSD" : "0x8d87a65ba30e09357fa2edea2c80dbac296e5dec2b18287113500b902942929d::celer_coin_manager::BusdCoin",
    //"okx_USDCav" : "0x39d84c2af3b0c9895b45d4da098049e382c451ba63bec0ce0396ff7af4bb5dff::coin::T",
    //"okx_USDTbs" : "0xacd014e8bdf395fa8497b6d585b164547a9d45269377bdf67c96c541b7fec9ed::coin::T",
    //"okx_WAVAX" : "0x5b1bbc25524d41b17a95dac402cf2f584f56400bf5cc06b53c36b331b1ec6e8f::coin::T",
    //"okx_ceWBTC" : "0x8d87a65ba30e09357fa2edea2c80dbac296e5dec2b18287113500b902942929d::celer_coin_manager::WbtcCoin",
    //"okx_DAI" : "0x407a220699982ebb514568d007938d2447d33667e4418372ffec1ddb24491b6c::coin::T",
    //"okx_CELO" : "0xac0c3c35d50f6ef00e3b4db6998732fe9ed6331384925fe8ec95fcd7745a9112::coin::T",
    //"okx_NEAR" : "0x394205c024d8e932832deef4cbfc7d3bb17ff2e9dc184fa9609405c2836b94aa::coin::T",
    //"okx_SUSHI" : "0x2305dd96edd8debb5a2049be54379c74e61b37ceb54a49bd7dee4726d2a6b689::coin::T",
    "okx_MEE" : "0xe9c192ff55cffab3963c695cff6dbf9dad6aff2bb5ac19a6415cad26a81860d9::mee_coin::MeeCoin",
    //"okx_DTO" : "0xd11107bdf0d6d7040c6c0bfbdecb6545191fdf13e8d8d259952f53e1713f61b5::ditto_discount_coin::DittoDiscountCoin",
    //"okx_APC" : "0x777821c78442e17d82c3d7a371f42de7189e4248e529fe6eee6bca40ddbb::apcoin::ApCoin",
    //"okx_EON" : "0x389dbbc6884a1d5b1ab4e1df2913a8c1b01251e50aed377554372b2842c5e3ef::EONcoin::EONCoin",
    "okx_ALT" : "0xd0b4efb4be7c3508d9a26a9b5405cf9f860d0b9e5fe2f498b90e68b8d2cedd3e::aptos_launch_token::AptosLaunchToken",
    //"okx_SWEAT" : "0x9aa4c03344444b53f4d9b1bca229ed2ac47504e3ea6cd0683ebdc0c5ecefd693::coin::T",
    //"okx_NEXM" : "0x1f9dca8eb42832b9ea07a804d745ef08833051e0c75c45b82665ef6f6e7fac32::coin::T",
    //"okx_FTT" : "0x419d16ebaeda8dc374b1178a61d24fb699799d55a3f475f427998769c537b51b::coin::T",
    //"okx_XCN" : "0xcefd39b563951a9ec2670aa57086f9adb3493671368ea60ff99e0bc98f697bb5::coin::T",
    //"okx_XBTC" : "0x3b0a7c06837e8fbcce41af0e629fdc1f087b06c06ff9e86f81910995288fd7fb::xbtc::XBTC",
  // Add more coin configurations as needed
};

// Initialize an array to hold JSON data
let jsonData = [];

// Function to fetch data based on source and ID
async function fetchData(source, id) {
  const url = `${URL_PREFIXES[source]}${id}`;
  try {
    const response = await fetch(url);
    const data = await response.json();
    return data;
  } catch (err) {
    console.error(`Error fetching data for ID: ${id}`, err);
    return null;
  }
}

// Function to process each coin
async function processCoin(coinSource, id) {
  const coin = coinSource.split('_')[1];
  const source = coinSource.split('_')[0];
  const dexConfig = DEX_CONFIG[source];

  if (!dexConfig) {
    console.error(`Error: DEX configuration missing for source ${source}`);
    return;
  }

  const tokenLogoUrlPath = dexConfig.match(/tokenLogoUrl=([^;]*)/)[1];
  const decimalsPath = dexConfig.match(/decimals=([^;]*)/)[1];
  const tokenNamePath = dexConfig.match(/tokenName=([^;]*)/)[1];
  const tokenSymbolPath = dexConfig.match(/tokenSymbol=([^;]*)/)[1];
  // Fetch data from API
  const data = await fetchData(source, id);
  if (!data) {
    console.error(`Error fetching data for ${coin}`);
    return;
  }

  function getNestedProperty(obj, path) {
    return path.split('.').reduce((acc, part) => acc && acc[part], obj);
  }

  // Extract values using JSON path (you can use a library like `jsonpath` for more complex paths)
  const tokenLogoUrl = getNestedProperty(data, tokenLogoUrlPath);
  const decimals = getNestedProperty(data, decimalsPath);
  const tokenName = getNestedProperty(data, tokenNamePath);
  const tokenSymbol = getNestedProperty(data, tokenSymbolPath);

  // Prepare the JSON document for MongoDB
  const json = {
    _id: coinSource,
    coin_id: id,
    tokenLogoUrl: tokenLogoUrl,
    decimals: decimals,
    tokenName: tokenName,
    tokenSymbol: tokenSymbol
  };
  
  jsonData.push(json);
  console.log(`Processed coin: ${coin}, source: ${source}`);
}

// Iterate over each coin configuration
async function processAllCoins() {
  for (const coin in DEX_COIN_CONFIGS) {
    await processCoin(coin, DEX_COIN_CONFIGS[coin]);
  }

  // Create MongoDB bulk operations
  const bulkOperations = jsonData.map(doc => ({
    updateOne: {
      filter: { _id: doc._id },
      update: { $set: doc },
      upsert: true
    }
  }));

  // Perform bulkWrite operation directly within the MongoDB client
  db = db.getSiblingDB('transactions');
  db.getCollection(process.env.MONGO_COLLECTION).bulkWrite(bulkOperations);

  console.log('Data imported successfully');
}

// Run the script to process all coins
processAllCoins().then(() => {
  const endTime = new Date();
  console.log(`Data import completed. Start time: ${startTime}, End time: ${endTime}`);
});

