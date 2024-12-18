// GridBackground.vue
<template>
    <canvas ref="canvasRef" />
</template>

<script lang="ts" setup>
import { ref, onMounted, onBeforeUnmount } from 'vue';

const GRID_SIZE = 100; // Size of each grid cell
const PULSE_INTERVAL = 4000; // New pulse every 4 seconds
const PULSE_SPEED = 1.2;
const POINT_BRIGHTNESS = .1;
const DOT_SIZE = 2;

interface Pulse {
  x: number;
  y: number;
  size: number;
  alpha: number;
  maxSize: number;
  speed: number;
}

interface GridPoint {
  x: number;
  y: number;
  brightness: number;
}

const canvasRef = ref<HTMLCanvasElement | null>(null);

onMounted(() => {
  const canvas = canvasRef.value;
  if (!canvas) return;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const pulses: Pulse[] = [];
  const gridPoints: GridPoint[] = [];

  const resize = () => {
    if (!canvas) return;
    canvas.width = canvas.offsetWidth;
    canvas.height = canvas.offsetHeight;
    initializeGrid();
  };

  const initializeGrid = () => {
    gridPoints.length = 0;
    for (let x = 0; x < canvas.width; x += GRID_SIZE) {
      for (let y = 0; y < canvas.height; y += GRID_SIZE) {
        gridPoints.push({
          x,
          y,
          brightness: 0
        });
      }
    }
  };

  const createPulse = () => {
    const randomGridPoint = gridPoints[Math.floor(Math.random() * gridPoints.length)];
    pulses.push({
      x: randomGridPoint.x,
      y: randomGridPoint.y,
      size: 0,
      alpha: 1,
      maxSize: Math.max(canvas.width, canvas.height) * 0.2,
      speed: PULSE_SPEED
    });

    setTimeout(createPulse, PULSE_INTERVAL);
  };

  window.addEventListener('resize', resize);
  resize();
  createPulse();

  const animate = () => {
    if (!canvas || !ctx) return;

    // Clear canvas with slight fade effect
    ctx.fillStyle = 'rgba(248, 249, 250, .15)';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Draw grid lines first (more subtle, gray color)
    ctx.strokeStyle = 'rgba(200, 200, 200, 0.08)'; // Light gray with low opacity
    ctx.lineWidth = 1;

    // Vertical lines
    for (let x = 0; x < canvas.width; x += GRID_SIZE) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, canvas.height);
      ctx.stroke();
    }

    // Horizontal lines
    for (let y = 0; y < canvas.height; y += GRID_SIZE) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(canvas.width, y);
      ctx.stroke();
    }

    // Update and draw grid points
    gridPoints.forEach(point => {
      // Reset brightness
      point.brightness = 0;

      // Check if any pulse affects this point
      pulses.forEach(pulse => {
        const dx = point.x - pulse.x;
        const dy = point.y - pulse.y;
        const distance = Math.sqrt(dx * dx + dy * dy);
        
        if (distance < pulse.size) {
          const impact = 1 - (distance / pulse.size);
          point.brightness = Math.max(point.brightness, impact * pulse.alpha);
        }
      });

      // Draw grid point (green, slightly larger)
      ctx.beginPath();
      ctx.arc(point.x, point.y, DOT_SIZE, 0, Math.PI * 2);
      // Base color more visible, enhanced by pulse brightness
      ctx.fillStyle = `rgba(72, 187, 120, ${POINT_BRIGHTNESS})`;
      ctx.fill();
    });

    // Update and draw pulses
    for (let i = pulses.length - 1; i >= 0; i--) {
      const pulse = pulses[i];
      pulse.size += pulse.speed;
      pulse.alpha = 1 - (pulse.size / pulse.maxSize);

      if (pulse.alpha <= 0) {
        pulses.splice(i, 1);
        continue;
      }

      // Draw pulse (green, maintaining visibility)
      ctx.beginPath();
      ctx.arc(pulse.x, pulse.y, pulse.size, 0, Math.PI * 2);
      ctx.strokeStyle = `rgba(72, 187, 120, ${pulse.alpha * 0.4})`;
      ctx.lineWidth = 2;
      ctx.stroke();
    }

    requestAnimationFrame(animate);
  };

  animate();

  onBeforeUnmount(() => {
    window.removeEventListener('resize', resize);
  });
});
</script>

<style scoped lang="scss">
canvas {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
}
</style>