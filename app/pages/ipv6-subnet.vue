<template>
	<LayoutTile
		title="IPv6 子网计算器"
		description="输入 IPv6 地址与 CIDR，获取网络信息并预览前 999 个地址。"
	>
		<div class="space-y-8">
			<UForm :state="formState" :schema="schema" class="space-y-6" @submit="calculate">
				<UFormField label="IPv6 / CIDR" name="cidrInput">
					<UInput v-model="formState.cidrInput" placeholder="2001:db8:85a3::8a2e:370:7334/64" size="lg" />
				</UFormField>

				<div class="flex justify-end">
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
						结果总览
					</h3>
					<UTable :data="summaryRows" :columns="summaryColumns" :ui="tableUi" />
				</section>

				<section class="space-y-4">
					<h3 class="text-xl font-semibold">
						{{ addressSectionTitle }}
					</h3>
					<p class="text-sm text-(--ui-text-muted)">
						{{ addressSectionDescription }}
					</p>
					<UTable :data="addressRows" :columns="addressColumns" :ui="tableUi" />
				</section>
			</div>
		</div>
	</LayoutTile>
</template>

<script lang="ts" setup>
	import LayoutTile from "~/components/Layout/Tile.vue";

	definePageMeta({
		name: "IPv6 子网计算器",
		icon: "lucide:network",
		description: "IPv6 子网计算工具",
		category: "tools"
	});

	interface SummaryRow {
		label: string
		value: string
	}

	interface Ipv6Result {
		ipv6Address: string
		fullIpv6Address: string
		totalIpAddresses: string
		networkAddress: string
		networkAddressFull: string
		ipRange: string
		addresses: string[]
	}

	const schema = z.object({
		cidrInput: z.string({
			error: "请输入 IPv6/CIDR"
		}).trim().nonempty("请输入 IPv6/CIDR，例如 2001:db8::1/64").refine(isValidIpv6Cidr, "请输入合法的 IPv6/CIDR")
	});

	type Schema = zInfer<typeof schema>;

	const formState = reactive<Schema>({
		cidrInput: "2001:db8:85a3::8a2e:370:7334/64"
	});

	const result = ref<Ipv6Result | null>(null);
	const errorMessage = ref("");
	const isCalculating = ref(false);

	const summaryColumns = [
		{ accessorKey: "label", header: "字段", id: "label" },
		{ accessorKey: "value", header: "值", id: "value" }
	];

	const addressColumns = computed(() => {
		if (!result.value) return [];
		const addressCount = result.value.addresses.length;
		const columnCount = addressCount <= 1 ? 1 : addressCount <= 10 ? 1 : 3;
		return Array.from({ length: columnCount }, (_, idx) => ({
			accessorKey: `column${idx + 1}`,
			header: "地址",
			id: `column${idx + 1}`
		}));
	});

	const addressSectionTitle = computed(() => {
		if (!result.value) return "地址列表";
		const count = result.value.addresses.length;
		return count >= 999 ? "前 999 个地址" : `共 ${count} 个地址`;
	});

	const addressSectionDescription = computed(() => {
		if (!result.value) return "地址以零填充形式展示，便于比对和复制。";
		const count = result.value.addresses.length;
		return count >= 999
			? "仅展示前 999 个地址，地址以零填充形式展示，便于比对和复制。"
			: "地址以零填充形式展示，便于比对和复制。";
	});

	const tableUi = {
		td: {
			base: "align-top whitespace-pre text-sm"
		}
	};

	const summaryRows = computed<SummaryRow[]>(() => {
		if (!result.value) return [];
		return [
			{ label: "IPv6 地址", value: result.value.ipv6Address },
			{ label: "完整 IPv6 地址", value: result.value.fullIpv6Address },
			{ label: "地址总数", value: result.value.totalIpAddresses },
			{ label: "网络地址", value: result.value.networkAddress },
			{ label: "网络地址（零填充）", value: result.value.networkAddressFull },
			{ label: "地址范围", value: result.value.ipRange }
		];
	});

	const addressRows = computed(() => {
		if (!result.value) return [];
		const addresses = result.value.addresses;
		const columns = addressColumns.value.length;
		const rowsNeeded = Math.ceil(addresses.length / columns);
		const formattedRows: Array<Record<string, string>> = [];

		for (let rowIndex = 0; rowIndex < rowsNeeded; rowIndex++) {
			const row: Record<string, string> = {};
			for (let colIndex = 0; colIndex < columns; colIndex++) {
				const addressIndex = rowIndex + colIndex * rowsNeeded;
				row[`column${colIndex + 1}`] = addressIndex < addresses.length ? addresses[addressIndex] : "";
			}
			formattedRows.push(row);
		}

		return formattedRows;
	});

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
			const response = await useTauriCoreInvoke<Ipv6Result>("compute_ipv6_subnet", {
				input: parsed.data.cidrInput.trim()
			});
			result.value = response;
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

	function isValidIpv6Cidr(value: string) {
		const normalized = value.replace(/\s+/g, "");
		const parts = normalized.split("/");
		if (parts.length !== 2) return false;
		const [ipPart, cidrPart] = parts;
		if (!isValidIpv6(ipPart)) return false;
		return /^\d{1,3}$/.test(cidrPart) && Number(cidrPart) >= 0 && Number(cidrPart) <= 128;
	}

	function isValidIpv6(value: string) {
		const trimmed = value.trim();
		if (!/^[0-9a-f:]+$/i.test(trimmed)) return false;
		if (trimmed.includes("::")) {
			if (trimmed.indexOf("::") !== trimmed.lastIndexOf("::")) return false;
		}
		const parts = trimmed.split(":").filter((part) => part.length > 0);
		return parts.length <= 8;
	}
</script>
