const [ ENV_PROD, ENV_DEV ] = [ 'PRODUCTION', 'DEVELOPMENT'];
const environment = '...';
function isUserAdmin(user) {
   if(environmentǃ=ENV_PROD){
       // bypass authZ checks in DEV
       return true;
   }
   return false;
}

// Fine in comment ǃ
"Fine in strings ǃ"
