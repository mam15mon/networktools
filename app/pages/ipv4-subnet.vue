<template>
	<LayoutTile
		title="IPv4 子网计算器"
		description="输入 IPv4 地址或 CIDR 网段，计算网络地址、广播地址、可用地址范围等详细信息。"
	>
		<div class="space-y-8">
			<UForm :state="formState" :schema="schema" class="space-y-6" @submit="calculate">
				<UFormField label="IPv4 / CIDR" name="cidrInput">
					<UInput v-model="formState.cidrInput" placeholder="例如：192.168.1.1/24" size="lg" />
				</UFormField>

				<div class="flex justify-between">
					<UButton
						type="button"
						variant="outline"
						size="lg"
						:loading="isDetectingIp"
						@click="detectPublicIp"
					>
						{{ isDetectingIp ? '检测中...' : '自动检测公网 IP' }}
					</UButton>
					<UButton type="submit" size="lg" :loading="isCalculating">
						计算
					</UButton>
				</div>
			</UForm>

			<UAlert
				v-if="errorMessage"
				variant="warning"
				icon="i-lucide-alert-triangle"
			>
				{{ errorMessage }}
			</UAlert>

			<div v-if="result" class="space-y-8">
				<section class="space-y-4">
					<h3 class="text-xl font-semibold">
						结果概览
					</h3>
					<UTable :data="summaryRows" :columns="summaryColumns" :ui="tableUi" />
				</section>

				<section class="space-y-4">
					<h3 class="text-xl font-semibold">
						同类子网列表
					</h3>
					<p class="text-sm text-(--ui-text-muted)">
						依据掩码粒度展示同类子网完整列表。
					</p>
					<p v-if="relatedNetworksHeading" class="text-sm font-medium">
						{{ relatedNetworksHeading }}
					</p>
					<UTable :data="relatedNetworkRows" :columns="networkColumns" :ui="tableUi" />
				</section>
			</div>
		</div>
	</LayoutTile>
</template>

<script lang="ts" setup>
	definePageMeta({
		name: "IP 子网计算器",
		icon: "lucide:network",
		description: "IPv4 子网信息计算工具",
		category: "tools"
	});

	// 页面加载时自动检测公网 IP
	onMounted(async () => {
		const { publicIp, isLoading, fetchPublicIp } = usePublicIp();

		// 如果已经有缓存的数据，直接使用
		if (publicIp.value && !isLoading.value) {
			formState.cidrInput = `${publicIp.value}/24`;
			await calculate();
		} else {
			// 没有缓存数据，请求获取
			try {
				await fetchPublicIp();
				if (publicIp.value) {
					formState.cidrInput = `${publicIp.value}/24`;
					await calculate();
				}
			} catch (error) {
				// 检测失败，使用默认值
				formState.cidrInput = "192.168.1.1/24";
				errorMessage.value = "自动检测公网 IP 失败，请手动输入 IPv4/CIDR 地址。";
			}
		}
	});

	interface SummaryRow {
		label: string
		value: string
	}

	interface NetworkRow {
		network: string
		range: string
		broadcast: string
	}

	interface SubnetResult {
		ipAddress: string
		networkAddress: string
		broadcastAddress: string
		subnetMask: string
		wildcardMask: string
		binarySubnetMask: string
		usableRange: string
		totalHosts: string
		usableHosts: string
		ipClass: string
		cidrNotation: string
		ipType: string
		shortNotation: string
		binaryId: string
		integerId: string
		hexId: string
		inAddrArpa: string
		ipv4MappedAddress: string
		sixToFourPrefix: string
	}

	interface SubnetResponse {
		result: SubnetResult
		relatedNetworks: NetworkRow[]
	}

	const schema = z.object({
		cidrInput: z.string({
			error: "请输入 IPv4/CIDR"
		}).trim().nonempty("请输入 IPv4/CIDR，例如 192.168.0.1/24").refine(isValidIpv4Cidr, "请输入合法的 IPv4/CIDR，例如 192.168.0.1/24")
	});

	type Schema = zInfer<typeof schema>;

	const formState = reactive<Schema>({
		cidrInput: "" // 初始为空，页面加载时自动检测
	});

	const result = ref<SubnetResult | null>(null);
	const errorMessage = ref("");
	const isCalculating = ref(false);
	const isDetectingIp = ref(false);

	const summaryColumns = [
		{ accessorKey: "label", header: "字段", id: "label" },
		{ accessorKey: "value", header: "值", id: "value" }
	];

	const networkColumns = [
		{ accessorKey: "network", header: "网络地址", id: "network" },
		{ accessorKey: "range", header: "可用主机范围", id: "range" },
		{ accessorKey: "broadcast", header: "广播地址", id: "broadcast" }
	];

	const tableUi = {
		td: {
			base: "align-top whitespace-pre-wrap"
		}
	};

	const summaryRows = computed<SummaryRow[]>(() => {
		if (!result.value) return [];
		return [
			{ label: "IP 地址", value: result.value.ipAddress },
			{ label: "网络地址", value: result.value.networkAddress },
			{ label: "可用主机范围", value: result.value.usableRange },
			{ label: "广播地址", value: result.value.broadcastAddress },
			{ label: "主机总数", value: result.value.totalHosts },
			{ label: "可用主机数", value: result.value.usableHosts },
			{ label: "子网掩码", value: result.value.subnetMask },
			{ label: "反掩码", value: result.value.wildcardMask },
			{ label: "二进制子网掩码", value: result.value.binarySubnetMask },
			{ label: "IP 类", value: result.value.ipClass },
			{ label: "CIDR 表示", value: result.value.cidrNotation },
			{ label: "IP 类型", value: result.value.ipType },
			{ label: "短格式", value: result.value.shortNotation },
			{ label: "二进制 ID", value: result.value.binaryId },
			{ label: "整数 ID", value: result.value.integerId },
			{ label: "十六进制 ID", value: result.value.hexId },
			{ label: "in-addr.arpa", value: result.value.inAddrArpa },
			{ label: "IPv4 映射地址", value: result.value.ipv4MappedAddress },
			{ label: "6to4 前缀", value: result.value.sixToFourPrefix }
		];
	});

	const relatedNetworkRows = computed<NetworkRow[]>(() => {
		if (!result.value) return [];
		const cidr = Number(result.value.cidrNotation.replace("/", ""));
		if (cidr === 24 || cidr >= 30 || cidr <= 16) {
			return [];
		}

		const ipInt = ipv4ToInt(result.value.ipAddress);
		const rows: NetworkRow[] = [];
		const step = 2 ** (32 - cidr);

		if (cidr >= 25 && cidr <= 29) {
			const base = ipInt & 0xFFFFFF00;
			const subnetCount = 1 << (cidr - 24);
			for (let index = 0; index < subnetCount; index++) {
				const networkInt = toUint32(base + index * step);
				rows.push(buildNetworkRow(networkInt, step, cidr));
			}
		} else if (cidr >= 17 && cidr <= 23) {
			const base = ipInt & 0xFFFF0000;
			const subnetCount = 1 << (cidr - 16);
			for (let index = 0; index < subnetCount; index++) {
				const networkInt = toUint32(base + index * step);
				rows.push(buildNetworkRow(networkInt, step, cidr));
			}
		}

		return rows;
	});

	const relatedNetworksHeading = computed(() => {
		if (!result.value) return "";
		const rows = relatedNetworkRows.value.length;
		if (!rows) return "";

		const cidr = Number(result.value.cidrNotation.replace("/", ""));
		const [oct1, oct2, oct3] = result.value.ipAddress.split(".");
		const scope = cidr >= 25 ? `${oct1}.${oct2}.${oct3}.*` : `${oct1}.${oct2}.*.*`;
		return `共 ${rows} 个 /${cidr} 子网，覆盖范围 ${scope}`;
	});

	const detectPublicIp = async () => {
		isDetectingIp.value = true;
		errorMessage.value = "";

		try {
			const { fetchPublicIp, getPublicIpCidr } = usePublicIp();

			// 强制重新获取（忽略缓存）
			await fetchPublicIp();

			const suggestedCidr = getPublicIpCidr(24);
			formState.cidrInput = suggestedCidr;

			// 自动触发计算
			await calculate();
		} catch (error) {
			const errorMsg = typeof error === "string"
				? error
				: (error as Error)?.message || "检测公网 IP 失败";

			errorMessage.value = `自动检测公网 IP 失败: ${errorMsg}。请手动输入 IPv4/CIDR 地址。`;
		} finally {
			isDetectingIp.value = false;
		}
	};

	const calculate = async () => {
		isCalculating.value = true;
		errorMessage.value = "";

		const parsed = schema.safeParse(formState);
		if (!parsed.success) {
			errorMessage.value = parsed.error.issues[0]?.message || "输入校验失败";
			isCalculating.value = false;
			return;
		}

		try {
			const response = await useTauriCoreInvoke<SubnetResponse>("compute_subnet", {
				input: parsed.data.cidrInput.trim()
			});
			result.value = response.result;
		} catch (error) {
			errorMessage.value = typeof error === "string"
				? error
				: (error as Error)?.message || "计算失败";
			result.value = null;
		} finally {
			isCalculating.value = false;
		}
	};

	onMounted(() => {
		void calculate();
	});

	function isValidIpv4Cidr(value: string) {
		const normalized = value.replace(/\s+/g, "");
		const parts = normalized.split("/");
		if (parts.length !== 2) return false;
		const [ipPart, cidrPart] = parts;
		if (!isValidIpv4(ipPart)) return false;
		return isValidCidr(cidrPart);
	}

	function isValidIpv4(value: string) {
		const parts = value.split(".");
		if (parts.length !== 4) return false;
		return parts.every((part) => {
			if (part === "") return false;
			if (!/^\d+$/.test(part)) return false;
			const num = Number(part);
			return num >= 0 && num <= 255;
		});
	}

	function isValidCidr(value: string) {
		if (!/^\d{1,2}$/.test(value)) return false;
		const num = Number(value);
		return num >= 0 && num <= 32;
	}

	function ipv4ToInt(ip: string) {
		return ip.split(".").reduce((acc, octet) => (acc << 8) + Number(octet), 0) >>> 0;
	}

	function intToIpv4(intValue: number) {
		return [24, 16, 8, 0].map((shift) => ((intValue >>> shift) & 0xFF).toString()).join(".");
	}

	function buildNetworkRow(networkInt: number, step: number, cidr: number): NetworkRow {
		const broadcastInt = toUint32(networkInt + step - 1);
		const firstHostInt = cidr >= 31 ? networkInt : networkInt + 1;
		const lastHostInt = cidr >= 31 ? broadcastInt : broadcastInt - 1;

		return {
			network: intToIpv4(networkInt),
			range: `${intToIpv4(firstHostInt)} - ${intToIpv4(lastHostInt)}`,
			broadcast: intToIpv4(broadcastInt)
		};
	}

	function toUint32(value: number) {
		return value >>> 0;
	}
</script>
