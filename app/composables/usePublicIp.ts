export const usePublicIp = () => {
	const publicIp = ref<string>("");
	const isLoading = ref(false);
	const error = ref<string>("");
	const lastFetched = ref<Date | null>(null);

	// 获取公网 IP
	const fetchPublicIp = async () => {
		// 如果最近 5 分钟内获取过，直接返回缓存
		if (lastFetched.value &&
			Date.now() - lastFetched.value.getTime() < 5 * 60 * 1000 &&
			publicIp.value) {
			return publicIp.value;
		}

		isLoading.value = true;
		error.value = "";

		try {
			const ip = await useTauriCoreInvoke<string>("get_public_ip");
			publicIp.value = ip;
			lastFetched.value = new Date();
			return ip;
		} catch (err) {
			const errorMsg = typeof err === "string"
				? err
				: (err as Error)?.message || "获取公网 IP 失败";
			error.value = errorMsg;
			throw errorMsg;
		} finally {
			isLoading.value = false;
		}
	};

	// 获取带默认前缀的 CIDR
	const getPublicIpCidr = (prefix: number = 24) => {
		if (!publicIp.value) return "";
		return `${publicIp.value}/${prefix}`;
	};

	return {
		publicIp: readonly(publicIp),
		isLoading: readonly(isLoading),
		error: readonly(error),
		fetchPublicIp,
		getPublicIpCidr
	};
};