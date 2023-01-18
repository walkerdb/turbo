Setup
  $ . ${TESTDIR}/../setup.sh
  $ . ${TESTDIR}/setup.sh $(pwd)
  $ ${TURBO} run build --skip-infer --concurrency=1
  \xe2\x80\xa2 Packages in scope: app-a, app-b (esc)
  \xe2\x80\xa2 Running build in 2 packages (esc)
  \xe2\x80\xa2 Remote caching disabled (esc)
  app-b:build: cache miss, executing 8cddca67a0b41357
  app-b:build: 
  app-b:build: > build
  app-b:build: > echo "building app-b" > lib/foo.txt && echo "building app-b" > dist/foo.txt
  app-b:build: 
  app-a:build: cache miss, executing 9424a149145e735d
  app-a:build: 
  app-a:build: > build
  app-a:build: > echo "building app-a" > lib/foo.txt && echo "building app-a" > dist/foo.txt
  app-a:build: 
  
   Tasks:    2 successful, 2 total
  Cached:    0 cached, 2 total
    Time:\s*[\.0-9]+m?s  (re)
  
  $ for f in $TARGET_DIR/node_modules/.cache/turbo/*.tar.zst; do tar -tf $f; done
  apps/app-b/.turbo/turbo-build.log
  apps/app-b/dist/
  apps/app-b/dist/.keep
  apps/app-b/dist/foo.txt
  apps/app-a/.turbo/turbo-build.log
  apps/app-a/lib/
  apps/app-a/lib/.keep
  apps/app-a/lib/foo.txt
