module.exports = {
  apps: [{
    name: 'maw',
    script: 'src/server.ts',
    interpreter: '/home/nat/.bun/bin/bun',
    env: {
      MAW_HOST: 'local',
      MAW_PORT: '3456',
    },
  }],
};
