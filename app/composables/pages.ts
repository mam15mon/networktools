export const usePages = () => {
	const router = useRouter();
	const currentRoute = useRoute();
	const { pageCategories } = useAppConfig();

	const pages = computed(() => {
		const routes = router.getRoutes().filter((route) => route.name !== "index" && route.name !== "all");

		const categorizedRoutes = routes.reduce((acc, route) => {
			// 跳过隐藏的页面
			if (route.meta.hidden) return acc;

			const category = route.meta.category as string || "other";
			if (!category) return acc;

			if (!acc[category]) {
				acc[category] = {
					label: pageCategories[category as keyof typeof pageCategories]?.label,
					icon: pageCategories[category as keyof typeof pageCategories]?.icon || "i-lucide-folder",
					to: route.path,
					children: []
				};
			}

			acc[category].children.push({
				label: route.meta.name as string || route.name,
				description: route.meta.description as string,
				icon: route.meta.icon || "i-lucide-file",
				to: route.path
			});

			if (route.path === currentRoute.path) {
				acc[category].to = route.path;
			}

			return acc;
		}, {} as Record<string, any>);

		const preferredOrder = ["tools", "firewall", "other"];
		const ordered = preferredOrder
			.map((key) => categorizedRoutes[key])
			.filter(Boolean);
		const remaining = Object.entries(categorizedRoutes)
			.filter(([key]) => !preferredOrder.includes(key))
			.map(([, value]) => value);
		return [...ordered, ...remaining];
	});

	return {
		pages
	};
};
