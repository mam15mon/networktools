export default defineAppConfig({
	app: {
		name: "NetworkTools"
	},
	pageCategories: {
		tools: {
			label: "网络工具",
			icon: "lucide:wrench"
		},
		firewall: {
			label: "Firewall",
			icon: "lucide:shield"
		},
		other: {
			label: "批量配置",
			icon: "lucide:layers"
		}
	},
	ui: {
		colors: {
			primary: "green",
			neutral: "zinc"
		},
		button: {
			slots: {
				base: "cursor-pointer"
			}
		},
		formField: {
			slots: {
				root: "w-full"
			}
		},
		input: {
			slots: {
				root: "w-full"
			}
		},
		textarea: {
			slots: {
				root: "w-full",
				base: "resize-none"
			}
		},
		accordion: {
			slots: {
				trigger: "cursor-pointer",
				item: "md:py-2"
			}
		},
		navigationMenu: {
			slots: {
				link: "cursor-pointer"
			}
		}
	}
});
