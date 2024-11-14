module.exports = {
  apps: [{
    name: 'matcher-grpc',
    script: '/home/chat/matcher/target/release/matcher',
    args: ['--server'],
    env: {
      RUST_LOG: 'debug',
      RUST_BACKTRACE: '1'
    },
    watch: false,
    instances: 1,
    autorestart: true,
    max_memory_restart: '1G',
    log_date_format: 'YYYY-MM-DD HH:mm:ss',
    error_file: '/home/chat/matcher/logs/error.log',
    out_file: '/home/chat/matcher/logs/out.log'
  }]
};
