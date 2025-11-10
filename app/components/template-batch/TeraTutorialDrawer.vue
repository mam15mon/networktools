<template>
	<UDrawer v-model:open="open" title="Tera 模板教学" :description="`常用语法与示例`" direction="right">
		<UButton
			variant="ghost"
			color="primary"
			icon="i-lucide-graduation-cap"
			@click="open = true"
		>
			Tera 模板教学
		</UButton>
		<template #body>
			<div class="p-4 space-y-6 text-sm leading-relaxed text-(--ui-text-muted)">
				<section>
					<UPageList>
						<h3 class="text-base font-semibold text-(--ui-text)">变量与过滤器</h3>
						<p>使用 <code v-pre>{{ variable | filter }}</code> 组合变量与过滤逻辑。</p>
						<div class="space-y-2">
							<div class="flex items-center gap-2"><UKbd>upper</UKbd><span class="text-sm">全部大写，常用于接口描述</span></div>
							<div class="flex items-center gap-2"><UKbd>capitalize</UKbd><span class="text-sm">句首大写，如 "core router" → "Core router"</span></div>
							<div class="flex items-center gap-2"><UKbd>trim / trim_start / trim_end</UKbd><span class="text-sm text-(--ui-text-muted)">清除模板或 Excel 中的多余空白</span></div>
							<div class="flex items-center gap-2"><UKbd>replace</UKbd><span class="text-sm"><code>from=" ", to="_"</code> 替换空格，生成合法标识符</span></div>
						</div>
					</UPageList>
				</section>

				<section>
					<h3 class="text-base font-semibold text-(--ui-text)">循环</h3>
					<p>遍历数组/列表时使用 <code v-pre>{% for item in items %}</code>。</p>
					<TemplateBatchCodeSnippet class="mt-2" :code="snippets.loop" />
					<p class="mt-2 text-xs">Excel 中需为 <code>vlans</code> 列提供 JSON 数组，例如：</p>
					<TemplateBatchCodeSnippet class="mt-2" :code="snippets.vlanArray" />
				</section>

				<section>
					<UPageList>
						<h3 class="text-base font-semibold text-(--ui-text)">条件</h3>
						<p>可使用条件语句进行分支控制：</p>
						<div class="space-y-2">
							<div class="flex items-center gap-2"><UKbd>{% if %}</UKbd><span class="text-sm">条件为真时执行</span></div>
							<div class="flex items-center gap-2"><UKbd>{% elif %}</UKbd><span class="text-sm">额外条件检查</span></div>
							<div class="flex items-center gap-2"><UKbd>{% else %}</UKbd><span class="text-sm">以上条件都不满足时执行</span></div>
							<div class="flex items-center gap-2"><UKbd>{% endif %}</UKbd><span class="text-sm text-(--ui-text-muted)">结束条件块</span></div>
						</div>
						<TemplateBatchCodeSnippet class="mt-3" :code="snippets.condition" />
						<UAlert color="neutral" variant="subtle" class="mt-3">
							<template #description>Excel 中的 <code>device.role</code> 建议填 <code>core</code>、<code>distribution</code> 等枚举值</template>
						</UAlert>
					</UPageList>
				</section>

				<section>
					<UPageList>
						<header class="space-y-1">
							<h3 class="text-base font-semibold text-(--ui-text)">批量生成接口</h3>
							<p>结合 <code>range</code> 与循环，可快速生成连续端口配置：</p>
						</header>
						<TemplateBatchCodeSnippet class="mt-2" :code="snippets.portsSingleCard" />
						<p class="text-xs mt-3">如果需要跨板卡（如 1/0/x 与 3/0/x），可以嵌套循环：</p>
						<TemplateBatchCodeSnippet class="mt-2" :code="snippets.portsMultiCard" />
						<div class="mt-2 space-y-2">
							<div class="flex items-center gap-2"><UKbd>device.port_count</UKbd><span class="text-sm">填写端口总数，如 <code>42</code> 生成 1-42 连续接口</span></div>
							<div class="flex items-center gap-2"><UKbd>slot_range</UKbd><span class="text-sm text-(--ui-text-muted)">自定义板卡列表或上下限，适合 1/0/x ~ 3/0/x 这类结构</span></div>
						</div>
					</UPageList>
				</section>

				<section>
					<UPageList>
						<h3 class="text-base font-semibold text-(--ui-text)">Default 回退</h3>
						<p>使用 <code v-pre>{{ value | default(value=fallback) }}</code> 可在 Excel 留空时使用备用字段。</p>
						<TemplateBatchCodeSnippet class="mt-2" :code="snippets.defaultSpeed" />
						<div class="mt-2 space-y-2">
							<div class="flex items-center gap-2"><UKbd>interface.speed</UKbd><span class="text-sm">主要字段，为空时使用回退值</span></div>
							<div class="flex items-center gap-2"><UKbd>interface.max_speed</UKbd><span class="text-sm text-(--ui-text-muted)">备用字段</span></div>
						</div>
					</UPageList>
				</section>

				<section>
					<h3 class="text-base font-semibold text-(--ui-text)">宏：复用接口/邻居配置</h3>
					<p>在同一个模板文件中使用 <code v-pre>{% macro %}</code>，可以把重复的接口或 BGP 邻居片段封装起来，按需调用：</p>
					<TemplateBatchCodeSnippet class="mt-2" :code="snippets.macro" />
					<p class="mt-2 text-xs">调用示例（同一个宏可在多处复用）：</p>
					<TemplateBatchCodeSnippet class="mt-2" :code="snippets.macroCall" />
					<UAlert color="neutral" variant="subtle" class="mt-3">
						<template #description>例如需要为多台 PE 设备生成相同格式的接口/BGP 邻居配置时，只需在 Excel 中列出参数，模板里调用宏即可减少复制粘贴</template>
					</UAlert>
				</section>

				<section>
					<UPageList>
						<h3 class="text-base font-semibold text-(--ui-text)">临时变量与格式化</h3>
						<p>可以用 <code v-pre>{% set %}</code> 或过滤器构造动态字符串：</p>
						<TemplateBatchCodeSnippet class="mt-2" :code="snippets.setBlock" />
						<div class="mt-2 space-y-2">
							<div class="flex items-center gap-2"><UKbd>format</UKbd><span class="text-sm">格式化输出，如 <code v-pre>{{ vlan.id | format(value="%03d") }}</code></span></div>
							<div class="flex items-center gap-2"><UKbd>trim</UKbd><span class="text-sm text-(--ui-text-muted)">清理空白字符，<code v-pre>{{ desc | trim }}</code></span></div>
						</div>
					</UPageList>
				</section>

				<section>
					<UPageList>
						<header class="space-y-1">
							<h3 class="text-base font-semibold text-(--ui-text)">复杂条件与默认值</h3>
							<p>组合 <code>if/elif/else</code> 与 <code>default</code> 可覆盖不同角色。</p>
						</header>
						<TemplateBatchCodeSnippet class="mt-2" :code="snippets.roleCondition" />
						<div class="mt-2 space-y-2">
							<div class="flex items-center gap-2"><UKbd>device.role</UKbd><span class="text-sm">设备角色枚举：<code>core</code>、<code>distribution</code>、<code>access</code></span></div>
							<div class="flex items-center gap-2"><UKbd>default</UKbd><span class="text-sm text-(--ui-text-muted)">结合 fallback 字段避免空值</span></div>
						</div>
					</UPageList>
				</section>

			</div>
		</template>
	</UDrawer>
</template>

<script setup lang="ts">
import { ref } from "vue";

const open = ref(false);
const snippets = {
	loop: `{% for vlan in vlans %}
interface Vlan{{ vlan.id }}
 description {{ vlan.name }}
{% endfor %}`,
	vlanArray: `[{"id":1,"name":"Core","ip":"10.0.0.1","mask":"255.255.255.0"}]`,
	condition: `{% if device.role == "core" %}
 router ospf 1
{% elif device.role == "distribution" %}
 ip route ...
{% else %}
 interface Vlan99
{% endif %}`,
	portsSingleCard: `{% for port in range(start=1, end=device.port_count + 1) %}
interface 100GE1/0/{{ port }}
 shutdown
{% endfor %}`,
	portsMultiCard: `{% for slot in range(start=1, end=device.slot_end + 1) %}
{% for port in range(start=0, end=device.port_per_slot) %}
interface 100GE{{ slot }}/0/{{ port }}
 shutdown
{% endfor %}
{% endfor %}`,
	defaultSpeed: `speed {{ interface.speed | default(value=interface.max_speed) }}`,
	macro: `{% macro bgp_neighbor(peer, asn, desc) %}
neighbor {{ peer }} remote-as {{ asn }}
neighbor {{ peer }} description {{ desc }}
{% endmacro %}`,
	macroCall: `{{ bgp_neighbor("10.1.1.2", 65001, "Core to PE1") }}`,
	setBlock: `{% set loopback_id = device.id + 100 %}
interface Loopback{{ loopback_id }}
 description {{ device.hostname | replace(from=" ", to="_") | upper }}`,
	roleCondition: `{% if device.role in ["core","distribution"] %}
 router ospf 1
{% elif device.role == "access" %}
 ip route 0.0.0.0 0.0.0.0 {{ device.uplink }}
{% endif %}`
};
</script>
