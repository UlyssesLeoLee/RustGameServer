import { useEffect, useState } from 'react';
import axios from 'axios';
import {
  Chart as ChartJS,
  LineElement,
  PointElement,
  LinearScale,
  TimeScale,
  Tooltip,
  Legend
} from 'chart.js';
import 'chartjs-adapter-date-fns';
import { Line } from 'react-chartjs-2';

ChartJS.register(LineElement, PointElement, LinearScale, TimeScale, Tooltip, Legend);

interface Point {
  x: number;
  y: number;
}

interface ActiveUsersChartProps {
  title?: string;
  refreshInterval?: number;
  authToken?: string;
}

export default function ActiveUsersChart({
  title = 'Active Users',
  refreshInterval = 5000,
  authToken,
}: ActiveUsersChartProps) {
  const [points, setPoints] = useState<Point[]>([]);

  useEffect(() => {
    let cancelled = false;
    const fetchActive = async () => {
      try {
        const res = await axios.get('/gm/metrics', {
          headers: authToken
            ? { Authorization: `Bearer ${authToken}` }
            : undefined,
          timeout: 5000,
        });
        const match = res.data.match(/online_connections\s+(\d+)/);
        if (match && !cancelled) {
          const count = Number(match[1]);
          setPoints(prev => [...prev.slice(-19), { x: Date.now(), y: count }]);
        }
      } catch {
        // ignore
      }
    };
    fetchActive();
    const id = setInterval(fetchActive, refreshInterval);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [refreshInterval, authToken]);

  const data = {
    datasets: [
      {
        label: 'Active Users',
        data: points,
        borderColor: 'rgb(75, 192, 192)',
        backgroundColor: 'rgba(75, 192, 192, 0.2)',
        tension: 0.3
      }
    ]
  };

  const options = {
    parsing: false,
    scales: {
      x: {
        type: 'time',
        time: {
          unit: 'minute'
        }
      },
      y: {
        beginAtZero: true,
        precision: 0
      }
    }
  } as const;

  return (
    <div>
      {title && <h3>{title}</h3>}
      <Line data={data} options={options} />
    </div>
  );
}
