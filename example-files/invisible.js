app.get('/network_health', async (req, res) => {
   const { timeout,ᅠ} = req.query;
   const checkCommands = [
       'ping -c 1 google.com',
       'curl -s http://example.com/',ᅠ
   ];
   // <handle response>
});
