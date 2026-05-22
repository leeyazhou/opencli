<template>
  <!-- 全局粒子背景画布容器 -->
  <div class="fixed inset-0 pointer-events-none z-0 overflow-hidden">
    <canvas ref="canvasRef" class="block w-full h-full"></canvas>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";

class Particle {
  public x = 0;
  public y = 0;
  public size = 0;
  public speedY = 0;
  public speedX = 0;
  public opacity = 0;
  private fadeRate = 0;
  private width = 0;
  private height = 0;

  constructor(width: number, height: number) {
    this.width = width;
    this.height = height;
    this.reset(true);
  }

  /**
   * 粒子物理状态重置
   * @param initial 是否是初始化阶段，若是，粒子随机分布在全屏；若否，重置在屏幕底部重新浮起
   */
  public reset(initial: boolean): void {
    this.x = Math.random() * this.width;
    this.y = initial ? Math.random() * this.height : this.height + 10;
    this.size = Math.random() * 1.5 + 0.5;
    this.speedY = -(Math.random() * 0.3 + 0.1);
    this.speedX = (Math.random() - 0.5) * 0.2;
    this.opacity = Math.random() * 0.4 + 0.1;
    this.fadeRate = Math.random() * 0.002 + 0.001;
  }

  /**
   * 刷新粒子物理坐标与淡入淡出属性
   */
  public update(width: number, height: number): void {
    this.width = width;
    this.height = height;

    this.x += this.speedX;
    this.y += this.speedY;
    this.opacity -= this.fadeRate;
    
    // 加一点微小的横向弦波抖动，产生自然的飘渺动感
    this.x += Math.sin(this.y * 0.02) * 0.05;

    if (this.opacity <= 0 || this.y < -10) {
      this.reset(false);
    }
  }

  /**
   * 在画布上绘制圆形粒子点
   */
  public draw(ctx: CanvasRenderingContext2D): void {
    ctx.beginPath();
    ctx.arc(this.x, this.y, this.size, 0, Math.PI * 2);
    // 使用科技紫蓝主色调 rgba(99, 102, 241, opacity)
    ctx.fillStyle = `rgba(99, 102, 241, ${this.opacity})`;
    ctx.fill();
  }
}

const canvasRef = ref<HTMLCanvasElement | null>(null);
let ctx: CanvasRenderingContext2D | null = null;
let animationFrameId = 0;
let width = 0;
let height = 0;
const PARTICLE_COUNT = 60;
const particles: Particle[] = [];

// 浏览器物理视口大小重算
const resize = () => {
  if (!canvasRef.value) return;
  width = canvasRef.value.width = window.innerWidth;
  height = canvasRef.value.height = window.innerHeight;
};

// Canvas 逐帧动画物理循环
const animate = () => {
  if (!ctx) return;

  // 每帧清空上一帧残留
  ctx.clearRect(0, 0, width, height);

  // 1. 物理计算与独立绘制每一个粒子
  for (const p of particles) {
    p.update(width, height);
    p.draw(ctx);
  }

  // 2. 空间连线物理计算：当两个粒子靠近到一定限度时，用半透明线连接它们，形成华丽的神经网络/星座星图连线特效
  for (let i = 0; i < particles.length; i++) {
    for (let j = i + 1; j < particles.length; j++) {
      const dx = particles[i].x - particles[j].x;
      const dy = particles[i].y - particles[j].y;
      const dist = Math.sqrt(dx * dx + dy * dy);

      if (dist < 80) {
        const alpha = (1 - dist / 80) * 0.06;
        ctx.beginPath();
        ctx.moveTo(particles[i].x, particles[i].y);
        ctx.lineTo(particles[j].x, particles[j].y);
        ctx.strokeStyle = `rgba(99, 102, 241, ${alpha})`;
        ctx.lineWidth = 0.5;
        ctx.stroke();
      }
    }
  }

  // 递归刷新下一帧
  animationFrameId = requestAnimationFrame(animate);
};

onMounted(() => {
  if (!canvasRef.value) return;
  ctx = canvasRef.value.getContext("2d");
  if (!ctx) return;

  resize();
  window.addEventListener("resize", resize);

  // 填充粒子群
  for (let i = 0; i < PARTICLE_COUNT; i++) {
    particles.push(new Particle(width, height));
  }

  // 启动动画循环
  animate();
});

onUnmounted(() => {
  window.removeEventListener("resize", resize);
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId);
  }
});
</script>

<style scoped>
/* 仅用于在 Canvas 之外没有其他交互事件的污染 */
canvas {
  user-select: none;
  pointer-events: none;
}
</style>
