import type { DailyActivity } from "../../lib/commands";

interface ActivityChartProps {
  activity: DailyActivity[];
  ariaLabel: string;
}

export function ActivityChart({ activity, ariaLabel }: ActivityChartProps) {
  const maximum = Math.max(1, ...activity.map((day) => day.attemptCount));
  return (
    <div aria-label={ariaLabel} className="pb-activity-chart" role="img">
      {activity.map((day) => (
        <div className="pb-activity-chart__column" key={day.localDate} title={`${day.localDate}: ${day.attemptCount}`}>
          <span className="pb-activity-chart__bar" style={{ height: `${(day.attemptCount / maximum) * 100}%` }} />
        </div>
      ))}
    </div>
  );
}
