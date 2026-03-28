export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set([]),
	mimeTypes: {},
	_: {
		client: {start:"_app/immutable/entry/start.Btd9XRYH.js",app:"_app/immutable/entry/app.BKiUcsk3.js",imports:["_app/immutable/entry/start.Btd9XRYH.js","_app/immutable/chunks/NFuQtVKp.js","_app/immutable/chunks/CUrHN2N4.js","_app/immutable/chunks/BSX7YyXe.js","_app/immutable/chunks/Bci9jIs1.js","_app/immutable/chunks/YLVhzCCS.js","_app/immutable/entry/app.BKiUcsk3.js","_app/immutable/chunks/CUrHN2N4.js","_app/immutable/chunks/BSX7YyXe.js","_app/immutable/chunks/Bci9jIs1.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
		nodes: [
			__memo(() => import('./nodes/0.js')),
			__memo(() => import('./nodes/1.js')),
			__memo(() => import('./nodes/2.js'))
		],
		remotes: {
			
		},
		routes: [
			{
				id: "/",
				pattern: /^\/$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 2 },
				endpoint: null
			}
		],
		prerendered_routes: new Set([]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();
