<template>
	<LayoutTile
		title="IP 地理位置查询"
		description="查询任意 IP 地址的地理位置信息，包括国家、省份、城市、运营商等详细信息。支持 IPv4 和 IPv6 地址。"
	>
		<div class="space-y-8">
			<UForm :state="formState" :schema="schema" class="space-y-6" @submit="lookupLocation">
				<UFormField label="IP 地址" name="ipInput">
					<UInput
						v-model="formState.ipInput"
						placeholder="例如：8.8.8.8 或 2001:4860:4860::8888"
						size="lg"
					/>
				</UFormField>

				<div class="flex justify-between">
					<UButton
						type="button"
						variant="outline"
						size="lg"
						:loading="isDetectingIp"
						@click="detectPublicIp"
					>
						{{ isDetectingIp ? '检测中...' : '使用我的公网 IP' }}
					</UButton>
					<UButton type="submit" size="lg" :loading="isLookingUp">
						查询位置
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

			<!-- API 状态信息 -->
			<UCard v-if="databaseInfo" class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-globe" class="size-5" />
						<h3 class="text-lg font-semibold">
							API 状态
						</h3>
					</div>
				</template>
				<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
					<div class="space-y-2">
						<p class="text-sm font-medium text-(--ui-text-muted)">
							API 端点
						</p>
						<p class="text-base text-sm truncate">
							{{ databaseInfo.apiEndpoint }}
						</p>
					</div>
					<div class="space-y-2">
						<p class="text-sm font-medium text-(--ui-text-muted)">
							IPv4 支持
						</p>
						<p class="text-base">
							{{ databaseInfo.supportsIpv4 ? '✅ 支持' : '❌ 不支持' }}
						</p>
					</div>
					<div class="space-y-2">
						<p class="text-sm font-medium text-(--ui-text-muted)">
							IPv6 支持
						</p>
						<p class="text-base">
							{{ databaseInfo.supportsIpv6 ? '✅ 支持' : '❌ 不支持' }}
						</p>
					</div>
					<div class="space-y-2">
						<p class="text-sm font-medium text-(--ui-text-muted)">
							数据来源
						</p>
						<p class="text-base">
							在线 API
						</p>
					</div>
				</div>
			</UCard>

			<!-- 查询结果 -->
			<div v-if="locationResult" class="space-y-8">
				<section class="space-y-4">
					<h3 class="text-xl font-semibold">
						查询结果
					</h3>
					<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
						<!-- IP 地址信息 -->
						<UCard class="bg-(--ui-bg)">
							<template #header>
								<div class="flex items-center gap-2">
									<Icon name="i-lucide-globe" class="size-5" />
									<h4 class="text-lg font-semibold">
										IP 地址信息
									</h4>
								</div>
							</template>
							<div class="space-y-4">
								<div class="space-y-2">
									<p class="text-sm font-medium text-(--ui-text-muted)">
										查询地址
									</p>
									<p class="text-base font-mono">
										{{ locationResult.ip }}
									</p>
								</div>
								<div class="space-y-2">
									<p class="text-sm font-medium text-(--ui-text-muted)">
										地址类型
									</p>
									<p class="text-base">
										{{ locationResult.isIpv4 ? 'IPv4' : 'IPv6' }}
									</p>
								</div>
							</div>
						</UCard>

						<!-- 地理位置 -->
						<UCard class="bg-(--ui-bg)">
							<template #header>
								<div class="flex items-center gap-2">
									<Icon name="i-lucide-map-pin" class="size-5" />
									<h4 class="text-lg font-semibold">
										地理位置
									</h4>
								</div>
							</template>
							<div class="space-y-4">
								<div class="space-y-2">
									<p class="text-sm font-medium text-(--ui-text-muted)">
										国家
									</p>
									<p class="text-base">
										{{ locationResult.location.country }}
									</p>
								</div>
								<div class="space-y-2">
									<p class="text-sm font-medium text-(--ui-text-muted)">
										省份/地区
									</p>
									<p class="text-base">
										{{ locationResult.location.region }}
									</p>
								</div>
								<div class="space-y-2">
									<p class="text-sm font-medium text-(--ui-text-muted)">
										城市
									</p>
									<p class="text-base">
										{{ locationResult.location.city }}
									</p>
								</div>
								<div class="space-y-2">
									<p class="text-sm font-medium text-(--ui-text-muted)">
										运营商
									</p>
									<p class="text-base">
										{{ locationResult.location.isp }}
									</p>
								</div>
							</div>
						</UCard>
					</div>
				</section>

				<!-- 数据来源说明 -->
				<div class="flex items-center gap-2 text-sm text-(--ui-text-muted)">
					<Icon name="i-lucide-info" class="size-4" />
					<span>数据来源：api.mir6.com JSON API | 实时数据，精度区县级</span>
				</div>
			</div>

			<!-- 使用说明 -->
			<UCard class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-help-circle" class="size-5" />
						<h3 class="text-lg font-semibold">
							使用说明
						</h3>
					</div>
				</template>
				<div class="space-y-4 text-sm">
					<div class="flex items-start gap-2">
						<Icon name="i-lucide-check" class="size-4 text-green-500 mt-0.5" />
						<div>
							<p class="font-medium">
								支持格式
							</p>
							<p class="text-(--ui-text-muted)">
								IPv4 地址（如 8.8.8.8）和 IPv6 地址（如 2001:4860:4860::8888）
							</p>
						</div>
					</div>
					<div class="flex items-start gap-2">
						<Icon name="i-lucide-check" class="size-4 text-green-500 mt-0.5" />
						<div>
							<p class="font-medium">
								自动检测
							</p>
							<p class="text-(--ui-text-muted)">
								点击"使用我的公网 IP"自动获取并查询当前公网 IP 地址
							</p>
						</div>
					</div>
					<div class="flex items-start gap-2">
						<Icon name="i-lucide-check" class="size-4 text-green-500 mt-0.5" />
						<div>
							<p class="font-medium">
								数据精度
							</p>
							<p class="text-(--ui-text-muted)">
								基于在线 API 实时查询，数据准确性和时效性更高
							</p>
						</div>
					</div>
				</div>
			</UCard>
		</div>
	</LayoutTile>
</template>

<script lang="ts" setup>
	definePageMeta({
		name: "IP 地理位置查询",
		icon: "lucide:map-pin",
		description: "查询 IP 地址的地理位置信息",
		category: "tools"
	});

	interface LocationResult {
		ip: string
		location: {
			country: string
			region: string
			city: string
			isp: string
			rawInfo: string
		}
		isIpv4: boolean
	}

	interface DatabaseInfo {
		apiEndpoint: string
		supportsIpv4: boolean
		supportsIpv6: boolean
	}

	const schema = z.object({
		ipInput: z.string({
			error: "请输入 IP 地址"
		}).trim().nonempty("请输入 IP 地址").refine(isValidIpAddress, "请输入合法的 IPv4 或 IPv6 地址，例如 8.8.8.8 或 2001:4860:4860::8888")
	});

	type Schema = zInfer<typeof schema>;

	const formState = reactive<Schema>({
		ipInput: "" // 初始为空，用户手动输入或自动检测
	});

	const locationResult = ref<LocationResult | null>(null);
	const databaseInfo = ref<DatabaseInfo | null>(null);
	const errorMessage = ref("");
	const isLookingUp = ref(false);
	const isDetectingIp = ref(false);

	// 加载数据库信息
	const loadDatabaseInfo = async () => {
		try {
			const info = await useTauriCoreInvoke<DatabaseInfo>("get_database_info");
			databaseInfo.value = info;
		} catch (error) {
			console.error("获取数据库信息失败:", error);
			errorMessage.value = "无法获取数据库信息，IP 定位功能可能不可用。";
		}
	};

	// 查询 IP 位置
	const lookupLocation = async () => {
		isLookingUp.value = true;
		errorMessage.value = "";

		const parsed = schema.safeParse(formState);
		if (!parsed.success) {
			errorMessage.value = parsed.error.issues[0]?.message || "输入校验失败";
			isLookingUp.value = false;
			return;
		}

		try {
			const result = await useTauriCoreInvoke<LocationResult>("lookup_ip_location", {
				ip: parsed.data.ipInput.trim()
			});

			locationResult.value = result;
		} catch (error) {
			console.error("IP 位置查询失败:", error);
			errorMessage.value = error instanceof Error ? error.message : "查询失败，请检查 IP 地址格式和数据库状态。";
		} finally {
			isLookingUp.value = false;
		}
	};

	// 自动检测公网 IP
	const detectPublicIp = async () => {
		isDetectingIp.value = true;
		errorMessage.value = "";

		try {
			const { publicIp, fetchPublicIp } = usePublicIp();
			await fetchPublicIp();

			if (publicIp.value) {
				formState.ipInput = publicIp.value;
				// 自动查询该 IP 的位置信息
				await lookupLocation();
			} else {
				errorMessage.value = "无法获取公网 IP 地址，请手动输入。";
			}
		} catch (error) {
			console.error("检测公网 IP 失败:", error);
			errorMessage.value = "检测公网 IP 失败，请手动输入 IP 地址。";
		} finally {
			isDetectingIp.value = false;
		}
	};

	function isValidIpAddress(value: string) {
		const trimmed = value.trim();

		// 尝试 IPv4 验证
		if (isValidIpv4(trimmed)) return true;

		// 尝试 IPv6 验证
		if (isValidIpv6(trimmed)) return true;

		return false;
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

	function isValidIpv6(value: string) {
		const trimmed = value.trim();
		if (!/^[0-9a-f:]+$/i.test(trimmed)) return false;
		if (trimmed.includes("::")) {
			if (trimmed.indexOf("::") !== trimmed.lastIndexOf("::")) return false;
		}
		const parts = trimmed.split(":").filter((part) => part.length > 0);
		return parts.length <= 8;
	}

	// 页面加载时获取数据库信息和初始化
	onMounted(async () => {
		await loadDatabaseInfo();

		// 预加载公网 IP
		const { publicIp, isLoading, fetchPublicIp } = usePublicIp();
		if (!isLoading.value && !publicIp.value) {
			try {
				await fetchPublicIp();
			} catch {
				// 静默处理错误
			}
		}
	});
</script>
