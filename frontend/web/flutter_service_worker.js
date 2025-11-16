'use strict';
const MANIFEST = 'flutter-app-manifest';
const TEMP = 'flutter-temp-cache';
const CACHE_NAME = 'flutter-app-cache';

const RESOURCES = {"assets/AssetManifest.bin": "49332a1a90d2cce57cbb4844b59f810b",
"assets/AssetManifest.bin.json": "bfa99878f3c8b44be71b234798ab5474",
"assets/AssetManifest.json": "bf2e5eb6523a07442921e06458837b0c",
"assets/assets/graphql/mutation/add_play_chain.graphql": "d8ce5ca952a66f339d5c6f199ecbb0a1",
"assets/assets/graphql/mutation/bet.graphql": "1f017ddd35351e15b3d7fe4682c0864b",
"assets/assets/graphql/mutation/deal_bet.graphql": "68c9a267ad9911363188740ebba7c436",
"assets/assets/graphql/mutation/exit_single_player_game.graphql": "23c59207380acafb5f6576c5547340f0",
"assets/assets/graphql/mutation/find_play_chain.graphql": "ba96019b6c26575a76022e2d38edcdef",
"assets/assets/graphql/mutation/get_balance.graphql": "c8e238d74055205f33c05b22f54d7a21",
"assets/assets/graphql/mutation/hit.graphql": "6459452e04fce6b0619f0d1e1f8db196",
"assets/assets/graphql/mutation/mint_token.graphql": "f2a41797a19c896a071caf187dc42ed8",
"assets/assets/graphql/mutation/request_table_seat.graphql": "f9c56571169d1bf50e6b0bc8dcf6fd29",
"assets/assets/graphql/mutation/stand.graphql": "05ad2469898f929d082ba859d13a6a34",
"assets/assets/graphql/mutation/start_single_player_game.graphql": "440942fa942057437dd199cb05049875",
"assets/assets/graphql/mutation/subscribe_to.graphql": "e1fcf4a5722eb3e13aa9a920d1e18dc2",
"assets/assets/graphql/mutation/unsubscribe_from.graphql": "d35832d72a1c7990dcc8ed074d00a0e9",
"assets/assets/graphql/query/get_balances.graphql": "c686097eda630f7911e7f501a69b88d5",
"assets/assets/graphql/query/get_deck.graphql": "49a528d3d2af00f47e8d65c4a7a18c11",
"assets/assets/graphql/query/get_play_chains.graphql": "01842809f7bf525a963eeb6629ea1d9b",
"assets/assets/graphql/query/get_profile.graphql": "5ed086ecb0b4205a56299525d945935f",
"assets/assets/graphql/query/get_user_status.graphql": "7c0ed754b4d33f1ba8beb80380bcfefe",
"assets/assets/graphql/query/multi_player_data.graphql": "e604b0b08c62052c6393f328648950c7",
"assets/assets/graphql/query/single_player_data.graphql": "6988e41914eaf3b6c3e0bbf23a0df5cd",
"assets/assets/graphql/subscription/notification.graphql": "bc8ff1b5aa036465ec01cae73b885c12",
"assets/assets/old-graphql/mutation/action.graphql": "35213359bc29c55508733e448c961a06",
"assets/assets/old-graphql/mutation/idle_action_check.graphql": "f699efcf1b8eebcac753d9f06ce55b1a",
"assets/assets/old-graphql/mutation/join.graphql": "f238279cc98af6e0ca14f8c498b3ad0c",
"assets/assets/old-graphql/query/get_chain_list.graphql": "e4687c324fb1a64e9b5b506d3b0d7c9e",
"assets/assets/old-graphql/query/get_game_room_status.graphql": "0471564b9a77c6e42a9b0b840f177fc6",
"assets/assets/old-graphql/query/get_history.graphql": "b4b0985a12a0297a507c5e7fc9ddd1cc",
"assets/assets/old-graphql/query/get_insight.graphql": "7a02de56e9cc0359571f2dc04388d1c1",
"assets/assets/old-graphql/query/get_leaderboard.graphql": "a58e5a9d86522d059d7ff6c075327bca",
"assets/assets/old-graphql/query/get_player_status.graphql": "4729e5e01993767ea01d7ae1333f7925",
"assets/assets/old-graphql/query/get_play_data.graphql": "1f8fd59e77a396edf3362c5586d8e481",
"assets/assets/old-graphql/query/get_user_status.graphql": "b15cbc4e390dbcdabc191e0e1b4624fc",
"assets/assets/old-graphql/query/single_player_data.graphql": "0859a279535c78a064acdcfc35a858ed",
"assets/FontManifest.json": "dc3d03800ccca4601324923c0b1d6d57",
"assets/fonts/MaterialIcons-Regular.otf": "6acf35a6a5da8b67dc30d0c957e4403e",
"assets/NOTICES": "f5041c0a5494f4ecc54babde8d8bd7b7",
"assets/packages/cupertino_icons/assets/CupertinoIcons.ttf": "33b7d9392238c04c131b6ce224e13711",
"assets/shaders/ink_sparkle.frag": "ecc85a2e95f5e9f53123dcaf8cb9b6ce",
"canvaskit/canvaskit.js": "140ccb7d34d0a55065fbd422b843add6",
"canvaskit/canvaskit.js.symbols": "58832fbed59e00d2190aa295c4d70360",
"canvaskit/canvaskit.wasm": "07b9f5853202304d3b0749d9306573cc",
"canvaskit/chromium/canvaskit.js": "5e27aae346eee469027c80af0751d53d",
"canvaskit/chromium/canvaskit.js.symbols": "193deaca1a1424049326d4a91ad1d88d",
"canvaskit/chromium/canvaskit.wasm": "24c77e750a7fa6d474198905249ff506",
"canvaskit/skwasm.js": "1ef3ea3a0fec4569e5d531da25f34095",
"canvaskit/skwasm.js.symbols": "0088242d10d7e7d6d2649d1fe1bda7c1",
"canvaskit/skwasm.wasm": "264db41426307cfc7fa44b95a7772109",
"canvaskit/skwasm_heavy.js": "413f5b2b2d9345f37de148e2544f584f",
"canvaskit/skwasm_heavy.js.symbols": "3c01ec03b5de6d62c34e17014d1decd3",
"canvaskit/skwasm_heavy.wasm": "8034ad26ba2485dab2fd49bdd786837b",
"favicon-16x16.png": "7cd1a2fcf05a42f87fca7d1a0c1e96af",
"favicon-32x32.png": "bcb010e34278987f98fdcd880cf08bb8",
"favicon.ico": "132f54fd80061359f9170721964d4071",
"favicon.png": "e5a0f7d71cdfd940705ba533ea03de11",
"flutter.js": "888483df48293866f9f41d3d9274a779",
"flutter_bootstrap.js": "82eee1baafb0150bff291771670793aa",
"fonts/Orbitron-Bold.ttf": "446368d913de79c000895e4b91dfb1af",
"icons/Icon-192.png": "9d3f5bce811180345ddbc89b46a75d89",
"icons/Icon-512.png": "4e55809289afc7fcac1fcc79a50023c0",
"icons/Icon-maskable-192.png": "9d3f5bce811180345ddbc89b46a75d89",
"icons/Icon-maskable-512.png": "4e55809289afc7fcac1fcc79a50023c0",
"index.html": "91612f3514ca85fcaf5c360dfb2f97ae",
"/": "91612f3514ca85fcaf5c360dfb2f97ae",
"main.dart.js": "e4afb0c7b39cd0058af03a3288623e9f",
"main.dart.mjs": "3c76eea4e8e9be83389d98121aa86650",
"main.dart.wasm": "d6ca07ba9c3012fdaa2dab3d23ce0f8e",
"manifest.json": "ff21955e07638e1632aa164a0ec03037",
"version.json": "12c46dee89c06cd2cd6a66990cb5a52a"};
// The application shell files that are downloaded before a service worker can
// start.
const CORE = ["main.dart.js",
"main.dart.wasm",
"main.dart.mjs",
"index.html",
"flutter_bootstrap.js",
"assets/AssetManifest.bin.json",
"assets/FontManifest.json"];

// During install, the TEMP cache is populated with the application shell files.
self.addEventListener("install", (event) => {
  self.skipWaiting();
  return event.waitUntil(
    caches.open(TEMP).then((cache) => {
      return cache.addAll(
        CORE.map((value) => new Request(value, {'cache': 'reload'})));
    })
  );
});
// During activate, the cache is populated with the temp files downloaded in
// install. If this service worker is upgrading from one with a saved
// MANIFEST, then use this to retain unchanged resource files.
self.addEventListener("activate", function(event) {
  return event.waitUntil(async function() {
    try {
      var contentCache = await caches.open(CACHE_NAME);
      var tempCache = await caches.open(TEMP);
      var manifestCache = await caches.open(MANIFEST);
      var manifest = await manifestCache.match('manifest');
      // When there is no prior manifest, clear the entire cache.
      if (!manifest) {
        await caches.delete(CACHE_NAME);
        contentCache = await caches.open(CACHE_NAME);
        for (var request of await tempCache.keys()) {
          var response = await tempCache.match(request);
          await contentCache.put(request, response);
        }
        await caches.delete(TEMP);
        // Save the manifest to make future upgrades efficient.
        await manifestCache.put('manifest', new Response(JSON.stringify(RESOURCES)));
        // Claim client to enable caching on first launch
        self.clients.claim();
        return;
      }
      var oldManifest = await manifest.json();
      var origin = self.location.origin;
      for (var request of await contentCache.keys()) {
        var key = request.url.substring(origin.length + 1);
        if (key == "") {
          key = "/";
        }
        // If a resource from the old manifest is not in the new cache, or if
        // the MD5 sum has changed, delete it. Otherwise the resource is left
        // in the cache and can be reused by the new service worker.
        if (!RESOURCES[key] || RESOURCES[key] != oldManifest[key]) {
          await contentCache.delete(request);
        }
      }
      // Populate the cache with the app shell TEMP files, potentially overwriting
      // cache files preserved above.
      for (var request of await tempCache.keys()) {
        var response = await tempCache.match(request);
        await contentCache.put(request, response);
      }
      await caches.delete(TEMP);
      // Save the manifest to make future upgrades efficient.
      await manifestCache.put('manifest', new Response(JSON.stringify(RESOURCES)));
      // Claim client to enable caching on first launch
      self.clients.claim();
      return;
    } catch (err) {
      // On an unhandled exception the state of the cache cannot be guaranteed.
      console.error('Failed to upgrade service worker: ' + err);
      await caches.delete(CACHE_NAME);
      await caches.delete(TEMP);
      await caches.delete(MANIFEST);
    }
  }());
});
// The fetch handler redirects requests for RESOURCE files to the service
// worker cache.
self.addEventListener("fetch", (event) => {
  if (event.request.method !== 'GET') {
    return;
  }
  var origin = self.location.origin;
  var key = event.request.url.substring(origin.length + 1);
  // Redirect URLs to the index.html
  if (key.indexOf('?v=') != -1) {
    key = key.split('?v=')[0];
  }
  if (event.request.url == origin || event.request.url.startsWith(origin + '/#') || key == '') {
    key = '/';
  }
  // If the URL is not the RESOURCE list then return to signal that the
  // browser should take over.
  if (!RESOURCES[key]) {
    return;
  }
  // If the URL is the index.html, perform an online-first request.
  if (key == '/') {
    return onlineFirst(event);
  }
  event.respondWith(caches.open(CACHE_NAME)
    .then((cache) =>  {
      return cache.match(event.request).then((response) => {
        // Either respond with the cached resource, or perform a fetch and
        // lazily populate the cache only if the resource was successfully fetched.
        return response || fetch(event.request).then((response) => {
          if (response && Boolean(response.ok)) {
            cache.put(event.request, response.clone());
          }
          return response;
        });
      })
    })
  );
});
self.addEventListener('message', (event) => {
  // SkipWaiting can be used to immediately activate a waiting service worker.
  // This will also require a page refresh triggered by the main worker.
  if (event.data === 'skipWaiting') {
    self.skipWaiting();
    return;
  }
  if (event.data === 'downloadOffline') {
    downloadOffline();
    return;
  }
});
// Download offline will check the RESOURCES for all files not in the cache
// and populate them.
async function downloadOffline() {
  var resources = [];
  var contentCache = await caches.open(CACHE_NAME);
  var currentContent = {};
  for (var request of await contentCache.keys()) {
    var key = request.url.substring(origin.length + 1);
    if (key == "") {
      key = "/";
    }
    currentContent[key] = true;
  }
  for (var resourceKey of Object.keys(RESOURCES)) {
    if (!currentContent[resourceKey]) {
      resources.push(resourceKey);
    }
  }
  return contentCache.addAll(resources);
}
// Attempt to download the resource online before falling back to
// the offline cache.
function onlineFirst(event) {
  return event.respondWith(
    fetch(event.request).then((response) => {
      return caches.open(CACHE_NAME).then((cache) => {
        cache.put(event.request, response.clone());
        return response;
      });
    }).catch((error) => {
      return caches.open(CACHE_NAME).then((cache) => {
        return cache.match(event.request).then((response) => {
          if (response != null) {
            return response;
          }
          throw error;
        });
      });
    })
  );
}
