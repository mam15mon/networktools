<template>
	<UContainer class="relative overflow-hidden h-screen">
		<div class="grid size-full place-content-center gap-y-8">
			<SvgoLogo :filled="true" :font-controlled="false" class="mx-auto size-40" />

			<div class="flex flex-col items-center gap-y-3">
				<UHeading :level="1" size="3xl" class="animate-pulse font-bold font-heading text-pretty sm:text-4xl">
					{{ app.name.toUpperCase() }}
				</UHeading>
			</div>
		</div>
	</UContainer>
</template>

<script lang="ts" setup>
	const { app } = useAppConfig();

	definePageMeta({
		layout: "home"
	});

	// 首页加载时预加载公网 IP
	const { fetchPublicIp } = usePublicIp();

	onMounted(async () => {
		// 异步预加载，不阻塞页面渲染
		fetchPublicIp().catch(() => {
			// 静默处理错误，不影响首页体验
		});
	});
</script>
