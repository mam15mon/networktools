<template>
	<LayoutTile
		title="IPv4 路由汇总"
		description="支持 CIDR、IP 范围与单个地址输入，可同时查看精确汇总与最小覆盖网段。"
	>
		<div class="space-y-8">
			<UForm :state="formState" :schema="schema" class="space-y-6" @submit="aggregate">
				<UFormField label="待汇总条目" name="entries">
					<UTextarea
						v-model="formState.entries"
						placeholder="例如：\n192.168.1.0/24\n192.168.1.10-192.168.1.20\n10.0.0.1"
						size="lg"
						:rows="8"
					/>
				</UFormField>

				<div class="flex justify-end gap-3">
					<UButton type="submit" size="lg" :loading="isAggregating">
						开始汇总
					</UButton>
					<UButton type="button" variant="ghost" @click="reset">
						清空
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
				<section class="space-y-3">
					<h3 class="text-xl font-semibold">
						输入概览
					</h3>
					<p class="text-sm text-(--ui-text-muted)">
						共解析 {{ result.normalizedInputs.length }} 条有效条目。
					</p>
					<UTable
						:columns="inputColumns"
						:data="inputRows"
						:ui="tableUi"
					/>
				</section>

				<section class="space-y-3">
					<h3 class="text-xl font-semibold">
						精确汇总
					</h3>
					<p class="text-sm text-(--ui-text-muted)">
						精确覆盖原始网段的最小 CIDR 集合。
					</p>
					<UTable
						:columns="preciseColumns"
						:data="preciseRows"
						:ui="tableUi"
					/>
				</section>

				<section class="space-y-3">
					<h3 class="text-xl font-semibold">
						非精确汇总
					</h3>
					<p class="text-sm text-(--ui-text-muted)">
						最小覆盖所有输入的单个 CIDR，可能包含额外地址。显示总地址数、输入地址数、额外地址数及其百分比。
					</p>
					<UTable
						:columns="approxColumns"
						:data="approxRows"
						:ui="tableUi"
					/>
				</section>

				<section v-if="result.errors.length" class="space-y-3">
					<h3 class="text-xl font-semibold text-(--ui-destructive)">
						解析失败
					</h3>
					<UAlert variant="danger" icon="i-lucide-octagon-alert">
						<ul class="list-disc pl-4 space-y-1">
							<li v-for="item in result.errors" :key="item">
								{{ item }}
							</li>
						</ul>
					</UAlert>
				</section>
			</div>
		</div>
	</LayoutTile>
</template>

<script lang="ts" setup>
	import LayoutTile from "~/components/Layout/Tile.vue";

	definePageMeta({
		name: "IPv4 路由汇总",
		icon: "lucide:git-merge",
		description: "IPv4 地址段汇总工具",
		category: "tools"
	});

	interface NonPreciseSummary {
		cidr: string
		totalAddresses: number
		inputAddresses: number
		extraAddresses: number
		extraPercentage: number
	}

	interface AggregateResult {
		normalizedInputs: string[]
		preciseSummary: string[]
		approximateSummary: string
		nonPreciseSummary?: NonPreciseSummary
		errors: string[]
	}

	const schema = z.object({
		entries: z.string({
			error: "请输入至少一条 IPv4 内容"
		}).trim().nonempty("请输入至少一条 IPv4 内容")
	});

	type Schema = zInfer<typeof schema>;

	const formState = reactive<Schema>({
		entries: "192.168.1.0/24\n192.168.1.100-200\n10.0.0.1"
	});

	const result = ref<AggregateResult | null>(null);
	const errorMessage = ref("");
	const isAggregating = ref(false);

	const tableUi = {
		td: {
			base: "align-top whitespace-pre text-sm"
		}
	};

	const inputColumns = [
		{ accessorKey: "value", header: "有效条目", id: "value" }
	];

	const preciseColumns = [
		{ accessorKey: "cidr", header: "CIDR", id: "cidr" }
	];

	const approxColumns = computed(() => {
		if (result.value?.nonPreciseSummary) {
			return [
				{ accessorKey: "cidr", header: "CIDR", id: "cidr" },
				{ accessorKey: "totalAddresses", header: "总地址数", id: "totalAddresses" },
				{ accessorKey: "inputAddresses", header: "输入地址数", id: "inputAddresses" },
				{ accessorKey: "extraAddresses", header: "额外地址数", id: "extraAddresses" },
				{ accessorKey: "extraPercentage", header: "额外地址百分比", id: "extraPercentage" }
			];
		}
		return [{ accessorKey: "cidr", header: "CIDR", id: "cidr" }];
	});

	const inputRows = computed(() => {
		if (!result.value) return [];
		return result.value.normalizedInputs.map((value) => ({ value }));
	});

	const preciseRows = computed(() => {
		if (!result.value) return [];
		return result.value.preciseSummary.map((cidr) => ({ cidr }));
	});

const formatPercentage = (value: number) => {
	if (value === 0) return "0%";
	if (value < 0.01) {
		return `${value.toFixed(4)}%`;
	}
	return `${value.toFixed(2)}%`;
};

const approxRows = computed(() => {
	if (!result.value) return [];

	// 如果有新的非精确汇总数据，使用它
	if (result.value.nonPreciseSummary) {
		const summary = result.value.nonPreciseSummary;
		return [{
			cidr: summary.cidr,
			totalAddresses: summary.totalAddresses.toLocaleString(),
			inputAddresses: summary.inputAddresses.toLocaleString(),
			extraAddresses: summary.extraAddresses.toLocaleString(),
			extraPercentage: formatPercentage(summary.extraPercentage)
		}];
	}

		// 否则回退到旧的简单显示
		return [{ cidr: result.value.approximateSummary }];
	});

	const aggregate = async () => {
		isAggregating.value = true;
		errorMessage.value = "";

		const parsed = schema.safeParse(formState);
		if (!parsed.success) {
			errorMessage.value = parsed.error.issues[0]?.message || "输入校验失败";
			isAggregating.value = false;
			return;
		}

		const entries = parsed.data.entries
			.split(/\r?\n|,/)
			.map((item) => item.trim())
			.filter((item) => item.length > 0);

		if (!entries.length) {
			errorMessage.value = "请输入至少一条 IPv4 内容";
			isAggregating.value = false;
			return;
		}

		try {
			const response = await useTauriCoreInvoke<AggregateResult>("aggregate_ipv4", {
				items: entries
			});
			result.value = response;
		} catch (error) {
			errorMessage.value = typeof error === "string"
				? error
				: (error as Error)?.message || "汇总失败";
			result.value = null;
		} finally {
			isAggregating.value = false;
		}
	};

	const reset = () => {
		formState.entries = "";
		result.value = null;
		errorMessage.value = "";
	};
</script>
