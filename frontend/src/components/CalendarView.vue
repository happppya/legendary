<script setup lang="ts">
// Calendar: month grid showing landmark target dates and milestone chips.
// Decorative for now (scheduling real landmark dates is a later milestone),
// but the month pager works.

import { computed, ref } from 'vue'
import { PhCaretLeft, PhCaretRight } from '@phosphor-icons/vue'

interface EventMark {
  day: number
  label: string
  kind: 'landmark' | 'release' | 'checkpoint'
}

const cur = ref(new Date(2026, 10, 1)) // November 2026 (playtest month)

const WEEKDAYS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']
const MONTHS = [
  'January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December',
]

// Fixed demo milestone marks (mirrors realm Landmarks/).
const MARKS: EventMark[] = [
  { day: 15, label: 'Steam Playtest Demo', kind: 'landmark' },
  { day: 9, label: 'Feature freeze', kind: 'checkpoint' },
  { day: 2, label: 'Build 0.14 cut', kind: 'release' },
  { day: 12, label: 'QA sweep', kind: 'checkpoint' },
  { day: 27, label: 'Playtest dry run', kind: 'checkpoint' },
]

const monthLabel = computed(() => {
  const d = cur.value
  return `${MONTHS[d.getMonth()]} ${d.getFullYear()}`
})

function shiftMonth(delta: number) {
  cur.value = new Date(cur.value.getFullYear(), cur.value.getMonth() + delta, 1)
}

function todayLabel(): string {
  return 'Today'
}

function isCurrentMonth(): boolean {
  const now = new Date()
  return now.getMonth() === cur.value.getMonth() && now.getFullYear() === cur.value.getFullYear()
}

const cells = computed<{ day: number | null; events: EventMark[] }[]>(() => {
  const year = cur.value.getFullYear()
  const month = cur.value.getMonth()
  const first = new Date(year, month, 1)
  const days = new Date(year, month + 1, 0).getDate()
  const out: { day: number | null; events: EventMark[] }[] = []
  for (let i = 0; i < first.getDay(); i++) out.push({ day: null, events: [] })
  for (let d = 1; d <= days; d++) {
    const events = MARKS.filter((m) => m.day === d)
    out.push({ day: d, events })
  }
  return out
})
</script>

<template>
  <section class="calendar" aria-label="Calendar">
    <header class="cal-head">
      <h1 class="cal-title">{{ monthLabel }}</h1>
      <div class="cal-nav">
        <button type="button" class="cal-btn" aria-label="Previous month" title="Previous month" @click="shiftMonth(-1)">
          <PhCaretLeft :size="13" aria-hidden="true" />
        </button>
        <button type="button" class="cal-btn today" :disabled="isCurrentMonth()" @click="cur = new Date()">
          {{ todayLabel() }}
        </button>
        <button type="button" class="cal-btn" aria-label="Next month" title="Next month" @click="shiftMonth(1)">
          <PhCaretRight :size="13" aria-hidden="true" />
        </button>
      </div>
    </header>

    <div class="cal-grid">
      <span v-for="wd in WEEKDAYS" :key="wd" class="cal-wd">{{ wd }}</span>
      <div
        v-for="(c, i) in cells"
        :key="i"
        class="cal-cell"
        :class="{ 'is-today': c.day === new Date().getDate() && isCurrentMonth(), 'has-events': c.events.length }"
      >
        <template v-if="c.day !== null">
          <span class="cal-day">{{ c.day }}</span>
          <div class="cal-events">
            <span v-for="(ev, j) in c.events" :key="j" class="cal-event" :class="ev.kind">
              {{ ev.label }}
            </span>
          </div>
        </template>
      </div>
    </div>
  </section>
</template>

<style scoped>
.calendar {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  min-height: 0;
  background: var(--bg-0);
}

.cal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 12px 18px;
  border-bottom: 1px solid var(--line-1);
}

.cal-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-1);
}

.cal-nav {
  display: flex;
  align-items: center;
  gap: 4px;
}

.cal-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 26px;
  padding: 0 9px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-1);
  color: var(--text-2);
  font-size: 11.5px;
}

.cal-btn:hover:not(:disabled) {
  background: var(--bg-2);
  color: var(--text-1);
}

.cal-btn:disabled {
  opacity: 0.5;
}

.cal-grid {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  grid-auto-rows: minmax(86px, 1fr);
  gap: 1px;
  background: var(--line-1);
  border-top: 1px solid var(--line-1);
  overflow-y: auto;
}

.cal-wd {
  background: var(--bg-1);
  padding: 7px 10px 0;
  font-size: 9.5px;
  font-weight: 600;
  letter-spacing: 0.09em;
  text-transform: uppercase;
  color: var(--faint);
}

.cal-cell {
  background: var(--bg-1);
  padding: 6px 8px;
  min-width: 0;
}

.cal-cell.has-events {
  background: var(--bg-2);
}

.cal-cell.is-today .cal-day {
  background: var(--gold);
  color: var(--on-gold);
  border-radius: 50%;
}

.cal-day {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  font-size: 11px;
  color: var(--text-2);
}

.cal-events {
  margin-top: 4px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.cal-event {
  display: block;
  padding: 2px 7px;
  border-radius: 999px;
  font-size: 9.5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-1);
  background: var(--chip-active-bg);
  border: 1px solid var(--chip-active-line);
  color: var(--chip-active-fg);
}

.cal-event.landmark {
  background: var(--sel-bg);
  border-color: var(--gold);
  color: var(--gold-strong);
}

.cal-event.release {
  background: var(--chip-unstarted-bg);
  border-color: var(--chip-unstarted-line);
  color: var(--chip-unstarted-fg);
}

.cal-event.checkpoint {
  background: var(--chip-blocked-bg);
  border-color: var(--chip-blocked-line);
  color: var(--chip-blocked-fg);
}
</style>
